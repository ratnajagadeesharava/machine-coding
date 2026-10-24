package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.StreamingService;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    @Test
    void shouldHashSurrogatePairsAsTwoUtf16CodeUnits() {
        StreamingService s = Candidate.of(Solution::create);
        s.createTopic("t", 7);

        // U+1F600 is the code units D83D DE00: 31*55357 + 56832 = 1772899 -> % 7 = 2 (UTF-8 bytes would give 4)
        assertThat(s.publish("t", "😀", "x")).isEqualTo("2:0");
    }

    @Test
    void shouldNotUseAbsoluteValueForTheMinimumHash() {
        StreamingService s = Candidate.of(Solution::create);
        s.createTopic("t4", 4);
        s.createTopic("t7", 7);

        // Integer.MIN_VALUE: floorMod 4 = 0, floorMod 7 = 5 (abs() would stay negative or give another partition)
        assertThat(s.publish("t4", "polygenelubricants", "x")).isEqualTo("0:0");
        assertThat(s.publish("t7", "polygenelubricants", "x")).isEqualTo("5:0");
    }

    @Test
    void shouldSplitSevenPartitionsAcrossThreeConsumers() {
        StreamingService s = Candidate.of(Solution::create);
        s.createTopic("t", 7);
        for (String c : List.of("c3", "c1", "c2")) {
            s.joinGroup("g", c, "t");
        }

        assertThat(s.assignment("g", "c1", "t")).containsExactly(0, 1, 2);
        assertThat(s.assignment("g", "c2", "t")).containsExactly(3, 4);
        assertThat(s.assignment("g", "c3", "t")).containsExactly(5, 6);
    }

    @Test
    void shouldOrderConsumerIdsAsPlainStrings() {
        StreamingService s = Candidate.of(Solution::create);
        s.createTopic("t", 3);
        for (String c : List.of("c10", "c2", "c1")) {
            s.joinGroup("g", c, "t");
        }

        // plain string order: c1 < c10 < c2
        assertThat(s.assignment("g", "c1", "t")).containsExactly(0);
        assertThat(s.assignment("g", "c10", "t")).containsExactly(1);
        assertThat(s.assignment("g", "c2", "t")).containsExactly(2);
    }

    @Test
    void shouldRejectACommitFromAConsumerThatLostThePartitionInARebalance() {
        StreamingService s = Candidate.of(Solution::create);
        s.createTopic("t", 2);
        s.joinGroup("g", "a", "t");
        s.publish("t", null, "x");
        s.publish("t", null, "y");
        s.commit("g", "a", "t", 1, 1);

        s.joinGroup("g", "b", "t");

        assertThatThrownBy(() -> s.commit("g", "a", "t", 1, 1)).isInstanceOf(IllegalStateException.class);
        s.commit("g", "b", "t", 1, 1);
        assertThat(s.committedOffset("g", "t", 1)).isEqualTo(1);
    }

    @Test
    void shouldLeaveStateUntouchedAfterFailedOperations() {
        StreamingService s = Candidate.of(Solution::create);
        s.createTopic("t", 2);
        s.joinGroup("g", "a", "t");
        s.publish("t", null, "x");

        assertThatThrownBy(() -> s.commit("g", "a", "t", 0, 9)).isInstanceOf(IllegalStateException.class);
        assertThatThrownBy(() -> s.publish("t", "k", null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.createTopic("t", 9)).isInstanceOf(IllegalArgumentException.class);

        assertThat(s.committedOffset("g", "t", 0)).isZero();
        assertThat(s.messageCount("t")).isEqualTo(1);
        assertThat(s.assignment("g", "a", "t")).containsExactly(0, 1);
        // a rejected publish must not consume a round-robin slot
        assertThat(s.publish("t", null, "y")).isEqualTo("1:0");
    }

    @Test
    void shouldRetainExactlyTheNewestMessagesUnderConcurrentPublishes() {
        StreamingService s = Candidate.of(Solution::create);
        s.createTopic("t", 1, 100);

        Concurrently.run(8, t -> {
            for (int i = 0; i < 2_000; i++) {
                s.publish("t", "same", "m");
            }
        });

        assertThat(s.latestOffset("t", 0)).isEqualTo(16_000);
        assertThat(s.earliestOffset("t", 0)).isEqualTo(15_900);
        assertThat(s.messageCount("t")).isEqualTo(100);
    }

    @Test
    void shouldDeliverEveryMessageExactlyOnceToTwoConsumersWhileProducersRun() {
        StreamingService s = Candidate.of(Solution::create);
        s.createTopic("t", 4);
        s.joinGroup("g", "a", "t");
        s.joinGroup("g", "b", "t");
        int producers = 4;
        int each = 1_000;
        AtomicInteger done = new AtomicInteger();

        List<List<String>> seen = Concurrently.collect(producers + 2, 60_000, t -> {
            List<String> mine = new ArrayList<>();
            if (t < producers) {
                for (int i = 0; i < each; i++) {
                    s.publish("t", i % 3 == 0 ? null : "k" + (i % 5), "p" + t + "-" + i);
                }
                done.incrementAndGet();
                return mine;
            }
            String me = t == producers ? "a" : "b";
            List<Integer> owned = s.assignment("g", me, "t");
            while (true) {
                boolean finished = done.get() == producers;
                List<String> batch = s.poll("g", me, "t", 50);
                for (String e : batch) {
                    String[] parts = e.split(":", 3);
                    mine.add(parts[2]);
                    s.commit("g", me, "t", Integer.parseInt(parts[0]), Long.parseLong(parts[1]) + 1);
                }
                if (batch.isEmpty() && finished) {
                    break;
                }
            }
            assertThat(owned).isNotEmpty();
            return mine;
        });

        Set<String> all = new HashSet<>();
        int count = 0;
        for (List<String> l : seen) {
            all.addAll(l);
            count += l.size();
        }
        assertThat(count).isEqualTo(producers * each);
        assertThat(all).hasSize(producers * each);
    }

    @Test
    void shouldKeepAssignmentsDisjointAndCompleteUnderConcurrentJoinsAndLeaves() {
        StreamingService s = Candidate.of(Solution::create);
        s.createTopic("t", 16);

        Concurrently.run(8, t -> {
            String me = "c" + t;
            for (int i = 0; i < 300; i++) {
                s.joinGroup("g", me, "t");
                s.assignment("g", me, "t");
                s.leaveGroup("g", me, "t");
            }
            s.joinGroup("g", me, "t");
        });

        Set<Integer> covered = new HashSet<>();
        for (int t = 0; t < 8; t++) {
            List<Integer> a = s.assignment("g", "c" + t, "t");
            assertThat(a).hasSize(2);
            for (int p : a) {
                assertThat(covered.add(p)).isTrue();
            }
        }
        assertThat(covered).hasSize(16);
    }
}

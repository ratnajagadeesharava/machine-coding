package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.StreamingService;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private StreamingService svc() {
        return Candidate.of(Solution::create);
    }

    private static StreamingService withTopic(StreamingService s, String topic, int partitions) {
        s.createTopic(topic, partitions);
        return s;
    }

    private static void bad(Runnable r) {
        assertThatThrownBy(r::run).isInstanceOf(IllegalArgumentException.class);
    }

    private static void state(Runnable r) {
        assertThatThrownBy(r::run).isInstanceOf(IllegalStateException.class);
    }

    @Test
    void shouldAssignGapFreeOffsetsPerPartition() {
        StreamingService s = withTopic(svc(), "t", 3);

        // keys "a","b","c" go to partitions 1,2,0 of 3 (97 % 3, 98 % 3, 99 % 3)
        assertThat(s.publish("t", "a", "m1")).isEqualTo("1:0");
        assertThat(s.publish("t", "b", "m2")).isEqualTo("2:0");
        assertThat(s.publish("t", "a", "m3")).isEqualTo("1:1");
        assertThat(s.publish("t", "c", "m4")).isEqualTo("0:0");
        assertThat(s.publish("t", "a", "m5")).isEqualTo("1:2");
        assertThat(s.latestOffset("t", 1)).isEqualTo(3);
        assertThat(s.latestOffset("t", 2)).isEqualTo(1);
        assertThat(s.latestOffset("t", 0)).isEqualTo(1);
    }

    @Test
    void shouldRoundRobinKeylessMessagesStartingAtPartitionZero() {
        StreamingService s = withTopic(svc(), "t", 3);

        List<String> got = new ArrayList<>();
        for (int i = 0; i < 7; i++) {
            got.add(s.publish("t", null, "m" + i));
        }

        assertThat(got).containsExactly("0:0", "1:0", "2:0", "0:1", "1:1", "2:1", "0:2");
    }

    @Test
    void shouldNotAdvanceRoundRobinForKeyedMessages() {
        StreamingService s = withTopic(svc(), "t", 3);

        assertThat(s.publish("t", null, "x")).isEqualTo("0:0");
        assertThat(s.publish("t", "b", "x")).isEqualTo("2:0");
        assertThat(s.publish("t", null, "x")).isEqualTo("1:0");
        assertThat(s.publish("t", "b", "x")).isEqualTo("2:1");
        assertThat(s.publish("t", null, "x")).isEqualTo("2:2");
    }

    @Test
    void shouldKeepRoundRobinCountersPerTopic() {
        StreamingService s = svc();
        s.createTopic("a", 2);
        s.createTopic("b", 2);

        assertThat(s.publish("a", null, "x")).isEqualTo("0:0");
        assertThat(s.publish("a", null, "x")).isEqualTo("1:0");
        assertThat(s.publish("b", null, "x")).isEqualTo("0:0");
    }

    @Test
    void shouldUsePolynomialHashOverTheKeyCharacters() {
        StreamingService s = withTopic(svc(), "t", 4);

        // "ab" = 97*31 + 98 = 3105 -> 3105 % 4 = 1 ; "" -> 0 ; single char "c" = 99 -> 3
        assertThat(s.publish("t", "ab", "x")).isEqualTo("1:0");
        assertThat(s.publish("t", "", "x")).isEqualTo("0:0");
        assertThat(s.publish("t", "c", "x")).isEqualTo("3:0");
    }

    @Test
    void shouldMapNegativeHashesWithFloorModulo() {
        StreamingService s = svc();
        s.createTopic("t3", 3);
        s.createTopic("t5", 5);

        // the 31-polynomial hash of this key is Integer.MIN_VALUE
        String key = "polygenelubricants";
        assertThat(s.publish("t3", key, "x")).isEqualTo("1:0");
        assertThat(s.publish("t5", key, "x")).isEqualTo("2:0");
    }

    @Test
    void shouldRejectBadTopicDefinitions() {
        StreamingService s = svc();
        s.createTopic("t", 2);

        bad(() -> s.createTopic("t", 2));
        bad(() -> s.createTopic("t", 5, 10));
        bad(() -> s.createTopic(" ", 2));
        bad(() -> s.createTopic(null, 2));
        bad(() -> s.createTopic("z", 0));
        bad(() -> s.createTopic("z", -1));
        bad(() -> s.createTopic("z", 2, 0));
        s.createTopic("z", 2);
        assertThat(s.messageCount("z")).isZero();
    }

    @Test
    void shouldRejectBadPublishes() {
        StreamingService s = withTopic(svc(), "t", 2);

        bad(() -> s.publish("nope", "k", "m"));
        bad(() -> s.publish(null, "k", "m"));
        bad(() -> s.publish("t", "k", null));
        assertThat(s.messageCount("t")).isZero();
        assertThat(s.publish("t", null, "")).isEqualTo("0:0");
    }

    @Test
    void shouldCountMessagesAcrossAllPartitions() {
        StreamingService s = withTopic(svc(), "t", 3);
        s.createTopic("other", 1);

        for (int i = 0; i < 10; i++) {
            s.publish("t", null, "m" + i);
        }
        s.publish("other", null, "x");

        assertThat(s.messageCount("t")).isEqualTo(10);
        assertThat(s.messageCount("other")).isEqualTo(1);
        assertThat(s.earliestOffset("t", 0)).isZero();
        assertThat(s.latestOffset("t", 0)).isEqualTo(4);
        assertThat(s.latestOffset("t", 1)).isEqualTo(3);
        bad(() -> s.messageCount("nope"));
        bad(() -> s.latestOffset("t", 3));
        bad(() -> s.latestOffset("t", -1));
        bad(() -> s.earliestOffset("t", 3));
    }

    @Test
    void shouldSplitPartitionsIntoContiguousRangesWithExtrasToTheFirstConsumers() {
        StreamingService s = withTopic(svc(), "t", 5);

        s.joinGroup("g", "c1", "t");
        assertThat(s.assignment("g", "c1", "t")).containsExactly(0, 1, 2, 3, 4);
        s.joinGroup("g", "c2", "t");
        assertThat(s.assignment("g", "c1", "t")).containsExactly(0, 1, 2);
        assertThat(s.assignment("g", "c2", "t")).containsExactly(3, 4);
        s.joinGroup("g", "c3", "t");
        assertThat(s.assignment("g", "c1", "t")).containsExactly(0, 1);
        assertThat(s.assignment("g", "c2", "t")).containsExactly(2, 3);
        assertThat(s.assignment("g", "c3", "t")).containsExactly(4);
    }

    @Test
    void shouldOrderConsumersByIdNotByJoinOrder() {
        StreamingService s = withTopic(svc(), "t", 4);

        s.joinGroup("g", "zed", "t");
        s.joinGroup("g", "amy", "t");
        s.joinGroup("g", "mia", "t");

        assertThat(s.assignment("g", "amy", "t")).containsExactly(0, 1);
        assertThat(s.assignment("g", "mia", "t")).containsExactly(2);
        assertThat(s.assignment("g", "zed", "t")).containsExactly(3);
    }

    @Test
    void shouldRebalanceWhenAConsumerLeaves() {
        StreamingService s = withTopic(svc(), "t", 6);
        s.joinGroup("g", "a", "t");
        s.joinGroup("g", "b", "t");
        s.joinGroup("g", "c", "t");
        assertThat(s.assignment("g", "b", "t")).containsExactly(2, 3);

        s.leaveGroup("g", "a", "t");

        assertThat(s.assignment("g", "b", "t")).containsExactly(0, 1, 2);
        assertThat(s.assignment("g", "c", "t")).containsExactly(3, 4, 5);
        state(() -> s.assignment("g", "a", "t"));
    }

    @Test
    void shouldLeaveSurplusConsumersIdle() {
        StreamingService s = withTopic(svc(), "t", 2);
        s.joinGroup("g", "a", "t");
        s.joinGroup("g", "b", "t");
        s.joinGroup("g", "c", "t");
        s.publish("t", null, "x");

        assertThat(s.assignment("g", "a", "t")).containsExactly(0);
        assertThat(s.assignment("g", "b", "t")).containsExactly(1);
        assertThat(s.assignment("g", "c", "t")).isEmpty();
        assertThat(s.poll("g", "c", "t", 10)).isEmpty();
    }

    @Test
    void shouldTreatRepeatedJoinAsNoOpAndRejectOutsiders() {
        StreamingService s = withTopic(svc(), "t", 2);
        s.joinGroup("g", "a", "t");
        s.joinGroup("g", "a", "t");
        s.joinGroup("g", "b", "t");

        assertThat(s.assignment("g", "a", "t")).containsExactly(0);
        state(() -> s.leaveGroup("g", "ghost", "t"));
        state(() -> s.poll("g", "ghost", "t", 1));
        state(() -> s.commit("g", "ghost", "t", 0, 0));
        state(() -> s.assignment("other-group", "a", "t"));
        bad(() -> s.joinGroup("g", "a", "nope"));
        bad(() -> s.joinGroup(" ", "a", "t"));
        bad(() -> s.joinGroup("g", null, "t"));
    }

    @Test
    void shouldPollPartitionsInOrderThenOffsetsAndHonourMax() {
        StreamingService s = withTopic(svc(), "t", 2);
        s.joinGroup("g", "c", "t");
        for (int i = 0; i < 6; i++) {
            s.publish("t", null, "m" + i); // p0: m0 m2 m4 ; p1: m1 m3 m5
        }

        assertThat(s.poll("g", "c", "t", 100))
                .containsExactly("0:0:m0", "0:1:m2", "0:2:m4", "1:0:m1", "1:1:m3", "1:2:m5");
        assertThat(s.poll("g", "c", "t", 4)).containsExactly("0:0:m0", "0:1:m2", "0:2:m4", "1:0:m1");
        assertThat(s.poll("g", "c", "t", 1)).containsExactly("0:0:m0");
    }

    @Test
    void shouldKeepColonsInMessagePayloads() {
        StreamingService s = withTopic(svc(), "t", 1);
        s.joinGroup("g", "c", "t");
        s.publish("t", null, "a:b:c");

        assertThat(s.poll("g", "c", "t", 5)).containsExactly("0:0:a:b:c");
    }

    @Test
    void shouldNotAdvanceTheCommittedOffsetWhenPolling() {
        StreamingService s = withTopic(svc(), "t", 1);
        s.joinGroup("g", "c", "t");
        for (int i = 0; i < 5; i++) {
            s.publish("t", null, "m" + i);
        }

        assertThat(s.poll("g", "c", "t", 2)).containsExactly("0:0:m0", "0:1:m1");
        assertThat(s.poll("g", "c", "t", 2)).containsExactly("0:0:m0", "0:1:m1");
        assertThat(s.committedOffset("g", "t", 0)).isZero();

        s.commit("g", "c", "t", 0, 2);

        assertThat(s.committedOffset("g", "t", 0)).isEqualTo(2);
        assertThat(s.poll("g", "c", "t", 10)).containsExactly("0:2:m2", "0:3:m3", "0:4:m4");
        s.commit("g", "c", "t", 0, 5);
        assertThat(s.poll("g", "c", "t", 10)).isEmpty();
    }

    @Test
    void shouldSeeMessagesPublishedAfterTheLastPoll() {
        StreamingService s = withTopic(svc(), "t", 1);
        s.joinGroup("g", "c", "t");
        s.publish("t", null, "m0");
        s.commit("g", "c", "t", 0, 1);
        assertThat(s.poll("g", "c", "t", 10)).isEmpty();

        s.publish("t", null, "m1");

        assertThat(s.poll("g", "c", "t", 10)).containsExactly("0:1:m1");
    }

    @Test
    void shouldPollOnlyTheAssignedPartitions() {
        StreamingService s = withTopic(svc(), "t", 2);
        s.joinGroup("g", "a", "t");
        s.joinGroup("g", "b", "t");
        for (int i = 0; i < 4; i++) {
            s.publish("t", null, "m" + i);
        }

        assertThat(s.poll("g", "a", "t", 10)).containsExactly("0:0:m0", "0:1:m2");
        assertThat(s.poll("g", "b", "t", 10)).containsExactly("1:0:m1", "1:1:m3");
    }

    @Test
    void shouldLetGroupsConsumeIndependently() {
        StreamingService s = withTopic(svc(), "t", 1);
        s.joinGroup("g1", "c", "t");
        s.joinGroup("g2", "c", "t");
        for (int i = 0; i < 3; i++) {
            s.publish("t", null, "m" + i);
        }

        s.commit("g1", "c", "t", 0, 3);

        assertThat(s.poll("g1", "c", "t", 10)).isEmpty();
        assertThat(s.poll("g2", "c", "t", 10)).containsExactly("0:0:m0", "0:1:m1", "0:2:m2");
        assertThat(s.committedOffset("g2", "t", 0)).isZero();
    }

    @Test
    void shouldKeepTopicMembershipsOfOneGroupSeparate() {
        StreamingService s = svc();
        s.createTopic("a", 2);
        s.createTopic("b", 1);
        s.joinGroup("g", "c1", "a");
        s.joinGroup("g", "c2", "a");
        s.joinGroup("g", "c1", "b");

        assertThat(s.assignment("g", "c1", "a")).containsExactly(0);
        assertThat(s.assignment("g", "c1", "b")).containsExactly(0);
        state(() -> s.assignment("g", "c2", "b"));
    }

    @Test
    void shouldRejectBackwardsAndPastTheEndCommits() {
        StreamingService s = withTopic(svc(), "t", 1);
        s.joinGroup("g", "c", "t");
        for (int i = 0; i < 4; i++) {
            s.publish("t", null, "m" + i);
        }
        s.commit("g", "c", "t", 0, 3);

        state(() -> s.commit("g", "c", "t", 0, 2));
        state(() -> s.commit("g", "c", "t", 0, 5));
        state(() -> s.commit("g", "c", "t", 0, -1));
        assertThat(s.committedOffset("g", "t", 0)).isEqualTo(3);
        s.commit("g", "c", "t", 0, 3); // same offset is fine
        s.commit("g", "c", "t", 0, 4); // exactly the log end is fine
        assertThat(s.committedOffset("g", "t", 0)).isEqualTo(4);
    }

    @Test
    void shouldRejectCommitsForPartitionsTheConsumerDoesNotOwn() {
        StreamingService s = withTopic(svc(), "t", 2);
        s.joinGroup("g", "a", "t");
        s.joinGroup("g", "b", "t");
        s.publish("t", null, "x");
        s.publish("t", null, "y");

        state(() -> s.commit("g", "a", "t", 1, 1));
        bad(() -> s.commit("g", "a", "t", 2, 0));
        bad(() -> s.commit("g", "a", "t", -1, 0));
        bad(() -> s.commit("g", "a", "nope", 0, 0));
        s.commit("g", "b", "t", 1, 1);
        assertThat(s.committedOffset("g", "t", 1)).isEqualTo(1);
    }

    @Test
    void shouldKeepCommittedOffsetsAcrossRebalances() {
        StreamingService s = withTopic(svc(), "t", 2);
        s.joinGroup("g", "a", "t");
        for (int i = 0; i < 6; i++) {
            s.publish("t", null, "m" + i);
        }
        s.commit("g", "a", "t", 0, 2);
        s.commit("g", "a", "t", 1, 1);

        s.joinGroup("g", "b", "t"); // a keeps 0, b takes 1
        assertThat(s.poll("g", "b", "t", 10)).containsExactly("1:1:m3", "1:2:m5");
        state(() -> s.commit("g", "a", "t", 1, 3));

        s.leaveGroup("g", "b", "t"); // a owns both again
        assertThat(s.poll("g", "a", "t", 10))
                .containsExactly("0:2:m4", "1:1:m3", "1:2:m5");
    }

    @Test
    void shouldKeepGroupOffsetsWhenEveryoneLeavesAndComesBack() {
        StreamingService s = withTopic(svc(), "t", 1);
        s.joinGroup("g", "a", "t");
        s.publish("t", null, "m0");
        s.publish("t", null, "m1");
        s.commit("g", "a", "t", 0, 1);
        s.leaveGroup("g", "a", "t");

        s.joinGroup("g", "z", "t");

        assertThat(s.poll("g", "z", "t", 10)).containsExactly("0:1:m1");
    }

    @Test
    void shouldRejectInvalidPollArguments() {
        StreamingService s = withTopic(svc(), "t", 1);
        s.joinGroup("g", "c", "t");

        bad(() -> s.poll("g", "c", "t", 0));
        bad(() -> s.poll("g", "c", "t", -3));
        bad(() -> s.poll("g", "c", "nope", 1));
        bad(() -> s.poll("g", " ", "t", 1));
        assertThat(s.poll("g", "c", "t", 1)).isEmpty();
    }

    @Test
    void shouldDropTheOldestMessagesBeyondTheRetentionLimit() {
        StreamingService s = svc();
        s.createTopic("t", 1, 3);
        s.joinGroup("g", "c", "t");

        for (int i = 0; i < 5; i++) {
            assertThat(s.publish("t", null, "m" + i)).isEqualTo("0:" + i);
        }

        assertThat(s.earliestOffset("t", 0)).isEqualTo(2);
        assertThat(s.latestOffset("t", 0)).isEqualTo(5);
        assertThat(s.messageCount("t")).isEqualTo(3);
        assertThat(s.poll("g", "c", "t", 10)).containsExactly("0:2:m2", "0:3:m3", "0:4:m4");
    }

    @Test
    void shouldApplyRetentionToEachPartitionSeparately() {
        StreamingService s = svc();
        s.createTopic("t", 2, 2);

        for (int i = 0; i < 6; i++) {
            s.publish("t", null, "m" + i); // 3 per partition
        }
        s.publish("t", "b", "k"); // partition 2 % 2 = 0 -> 4 in p0

        assertThat(s.earliestOffset("t", 0)).isEqualTo(2);
        assertThat(s.earliestOffset("t", 1)).isEqualTo(1);
        assertThat(s.messageCount("t")).isEqualTo(4);
    }

    @Test
    void shouldJumpToTheEarliestOffsetWhenTheCommittedOneWasDropped() {
        StreamingService s = svc();
        s.createTopic("t", 1, 2);
        s.joinGroup("g", "c", "t");
        s.publish("t", null, "m0");
        s.commit("g", "c", "t", 0, 1);
        for (int i = 1; i < 6; i++) {
            s.publish("t", null, "m" + i);
        }

        assertThat(s.committedOffset("g", "t", 0)).isEqualTo(1);
        assertThat(s.earliestOffset("t", 0)).isEqualTo(4);
        assertThat(s.poll("g", "c", "t", 10)).containsExactly("0:4:m4", "0:5:m5");
        s.commit("g", "c", "t", 0, 6);
        assertThat(s.poll("g", "c", "t", 10)).isEmpty();
    }

    @Test
    void shouldGiveConcurrentPublishersUniqueGapFreeOffsets() {
        StreamingService s = withTopic(svc(), "t", 4);
        int threads = 8;
        int each = 500;

        List<List<String>> results = Concurrently.collect(threads, t -> {
            List<String> mine = new ArrayList<>();
            for (int i = 0; i < each; i++) {
                String key = (i % 2 == 0) ? null : "key-" + (i % 7);
                mine.add(s.publish("t", key, "t" + t + "-" + i));
            }
            return mine;
        });

        Set<String> all = new HashSet<>();
        results.forEach(all::addAll);
        assertThat(all).hasSize(threads * each);
        long total = 0;
        for (int p = 0; p < 4; p++) {
            long latest = s.latestOffset("t", p);
            total += latest;
            for (long o = 0; o < latest; o++) {
                assertThat(all).contains(p + ":" + o);
            }
        }
        assertThat(total).isEqualTo(threads * each);
        assertThat(s.messageCount("t")).isEqualTo(threads * each);
    }

    @Test
    void shouldRoundRobinExactlyEvenlyUnderConcurrentKeylessPublishes() {
        StreamingService s = withTopic(svc(), "t", 4);

        Concurrently.run(8, t -> {
            for (int i = 0; i < 250; i++) {
                s.publish("t", null, "m");
            }
        });

        for (int p = 0; p < 4; p++) {
            assertThat(s.latestOffset("t", p)).isEqualTo(500);
        }
    }
}

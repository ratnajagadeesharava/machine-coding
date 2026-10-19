package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Replica;
import java.util.ArrayList;
import java.util.List;
import java.util.Random;
import java.util.Set;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private static Replica replica(String id) {
        return Candidate.of(() -> Solution.replica(id));
    }

    /** Both directions: afterwards a and b have seen everything either has seen. */
    private static void sync(Replica a, Replica b) {
        String fromA = a.snapshot();
        String fromB = b.snapshot();
        a.merge(fromB);
        b.merge(fromA);
    }

    @Test
    void shouldCountLocallyAndStartAtZero() {
        Replica a = replica("a");

        assertThat(a.counter("likes")).isZero();
        a.increment("likes", 5);
        a.decrement("likes", 2);
        a.increment("likes", 1);

        assertThat(a.counter("likes")).isEqualTo(4);
        assertThat(a.counter("other")).isZero();
    }

    @Test
    void shouldRejectNegativeAmounts() {
        Replica a = replica("a");

        assertThatThrownBy(() -> a.increment("c", -1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> a.decrement("c", -1)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldNotLoseConcurrentIncrementsOnDifferentReplicas() {
        Replica a = replica("a");
        Replica b = replica("b");

        a.increment("views", 5);
        b.increment("views", 3);
        b.decrement("views", 1);
        sync(a, b);

        assertThat(a.counter("views")).isEqualTo(7);
        assertThat(b.counter("views")).isEqualTo(7);
    }

    @Test
    void shouldNotCountTwiceWhenTheSameSnapshotIsMergedAgain() {
        Replica a = replica("a");
        Replica b = replica("b");
        a.increment("c", 10);
        String snap = a.snapshot();

        b.merge(snap);
        b.merge(snap);
        b.merge(a.snapshot());

        assertThat(b.counter("c")).isEqualTo(10);
    }

    @Test
    void shouldNotRevertNewerStateWhenAnOldSnapshotIsMerged() {
        Replica a = replica("a");
        Replica b = replica("b");
        a.increment("c", 1);
        String old = a.snapshot();
        a.increment("c", 4);
        b.merge(a.snapshot());

        b.merge(old);

        assertThat(b.counter("c")).isEqualTo(5);
    }

    @Test
    void shouldTakeSnapshotsByValue() {
        Replica a = replica("a");
        Replica b = replica("b");
        a.increment("c", 1);
        String snap = a.snapshot();
        a.increment("c", 100);
        a.add("s", "x");

        b.merge(snap);

        assertThat(b.counter("c")).isEqualTo(1);
        assertThat(b.contains("s", "x")).isFalse();
    }

    @Test
    void shouldAddRemoveAndReAddElements() {
        Replica a = replica("a");

        a.add("tags", "x");
        a.add("tags", "y");
        assertThat(a.elements("tags")).containsExactlyInAnyOrder("x", "y");

        a.remove("tags", "x");
        assertThat(a.contains("tags", "x")).isFalse();
        assertThat(a.elements("tags")).containsExactly("y");

        a.add("tags", "x");
        assertThat(a.contains("tags", "x")).isTrue();
        a.remove("tags", "nope");
        assertThat(a.elements("tags")).containsExactlyInAnyOrder("x", "y");
    }

    @Test
    void shouldReturnACopyOfTheElements() {
        Replica a = replica("a");
        a.add("s", "x");
        Set<String> copy = a.elements("s");

        a.add("s", "y");

        assertThat(copy).containsExactly("x");
    }

    @Test
    void shouldReplicateAdditionsAndRemovals() {
        Replica a = replica("a");
        Replica b = replica("b");
        a.add("s", "x");
        sync(a, b);
        assertThat(b.contains("s", "x")).isTrue();

        b.remove("s", "x");
        sync(a, b);

        assertThat(a.contains("s", "x")).isFalse();
        assertThat(b.contains("s", "x")).isFalse();
    }

    @Test
    void shouldLetAConcurrentAddWinOverARemove() {
        Replica a = replica("a");
        Replica b = replica("b");
        a.add("s", "x");
        sync(a, b);

        a.remove("s", "x");
        b.add("s", "x");
        sync(a, b);

        assertThat(a.contains("s", "x")).isTrue();
        assertThat(b.contains("s", "x")).isTrue();
    }

    @Test
    void shouldOnlyRemoveWhatTheRemoverHadObserved() {
        Replica a = replica("a");
        Replica b = replica("b");
        a.add("s", "x");
        b.remove("s", "x");
        sync(a, b);

        assertThat(a.contains("s", "x")).isTrue();
        assertThat(b.contains("s", "x")).isTrue();
    }

    @Test
    void shouldNotResurrectARemovedElementFromAStaleSnapshot() {
        Replica a = replica("a");
        Replica b = replica("b");
        a.add("s", "x");
        String stale = a.snapshot();
        b.merge(stale);
        b.remove("s", "x");
        a.merge(b.snapshot());

        b.merge(stale);
        a.merge(stale);

        assertThat(a.contains("s", "x")).isFalse();
        assertThat(b.contains("s", "x")).isFalse();
    }

    @Test
    void shouldKeepAReAddedElementAfterAnOldRemoveArrivesAgain() {
        Replica a = replica("a");
        Replica b = replica("b");
        a.add("s", "x");
        a.remove("s", "x");
        String afterRemove = a.snapshot();
        a.add("s", "x");
        b.merge(a.snapshot());

        b.merge(afterRemove);

        assertThat(b.contains("s", "x")).isTrue();
    }

    @Test
    void shouldKeepTheHighestTimestampInARegister() {
        Replica a = replica("a");

        assertThat(a.read("title")).isEmpty();
        a.write("title", "v1", 10);
        a.write("title", "v0", 5);
        assertThat(a.read("title")).hasValue("v1");
        a.write("title", "v2", 20);
        assertThat(a.read("title")).hasValue("v2");
    }

    @Test
    void shouldConvergeRegistersWhateverTheMergeOrder() {
        Replica a = replica("a");
        Replica b = replica("b");
        Replica c = replica("c");
        a.write("r", "from-a", 5);
        b.write("r", "from-b", 9);
        c.write("r", "from-c", 7);

        a.merge(b.snapshot());
        a.merge(c.snapshot());
        c.merge(a.snapshot());
        b.merge(c.snapshot());
        b.merge(a.snapshot());

        assertThat(a.read("r")).hasValue("from-b");
        assertThat(b.read("r")).hasValue("from-b");
        assertThat(c.read("r")).hasValue("from-b");
    }

    @Test
    void shouldBreakTimestampTiesByTheLargerReplicaId() {
        Replica a = replica("a");
        Replica b = replica("b");
        a.write("r", "from-a", 10);
        b.write("r", "from-b", 10);

        sync(a, b);

        assertThat(a.read("r")).hasValue("from-b");
        assertThat(b.read("r")).hasValue("from-b");
    }

    @Test
    void shouldKeepTheThreeTypesInSeparateNamespaces() {
        Replica a = replica("a");
        Replica b = replica("b");

        a.increment("x", 3);
        a.add("x", "e");
        a.write("x", "v", 1);
        sync(a, b);

        assertThat(b.counter("x")).isEqualTo(3);
        assertThat(b.elements("x")).containsExactly("e");
        assertThat(b.read("x")).hasValue("v");
        assertThat(b.counter("e")).isZero();
        assertThat(b.contains("v", "e")).isFalse();
    }

    @Test
    void shouldHandleAwkwardNamesAndValues() {
        Replica a = replica("a");
        Replica b = replica("b");
        String weird = "line1\nline2 | with ;,'\" and ünïcode ✓";

        a.add(weird, weird);
        a.write(weird, weird, 1);
        a.increment(weird, 2);
        b.merge(a.snapshot());

        assertThat(b.contains(weird, weird)).isTrue();
        assertThat(b.read(weird)).hasValue(weird);
        assertThat(b.counter(weird)).isEqualTo(2);
    }

    @Test
    void shouldMergeAnEmptyReplicaAndItself() {
        Replica a = replica("a");
        Replica empty = replica("e");
        a.increment("c", 2);
        a.add("s", "x");

        a.merge(empty.snapshot());
        a.merge(a.snapshot());
        empty.merge(a.snapshot());

        assertThat(a.counter("c")).isEqualTo(2);
        assertThat(empty.counter("c")).isEqualTo(2);
        assertThat(empty.elements("s")).containsExactly("x");
    }

    @Test
    void shouldNotLoseIncrementsFromManyThreadsOnOneReplica() {
        Replica a = replica("a");

        Concurrently.run(8, 20_000, t -> {
            for (int i = 0; i < 1_000; i++) {
                a.increment("c", 1);
                a.add("s", "t" + t);
                a.write("r", "v" + t, i);
            }
        });

        assertThat(a.counter("c")).isEqualTo(8_000);
        assertThat(a.elements("s")).hasSize(8);
        assertThat(a.read("r")).isPresent();
    }

    @Test
    void shouldConvergeAfterRandomOperationsAndRandomGossip() {
        for (long seed = 0; seed < 100; seed++) {
            Random rnd = new Random(seed);
            List<Replica> rs = List.of(replica("a"), replica("b"), replica("c"));
            long[] expected = new long[1];
            List<String> inFlight = new ArrayList<>();
            for (int step = 0; step < 150; step++) {
                int ts = rnd.nextInt(50) * 1000 + step;
                Replica r = rs.get(rnd.nextInt(3));
                switch (rnd.nextInt(8)) {
                    case 0, 1 -> {
                        long n = rnd.nextInt(5);
                        r.increment("c", n);
                        expected[0] += n;
                    }
                    case 2 -> {
                        long n = rnd.nextInt(5);
                        r.decrement("c", n);
                        expected[0] -= n;
                    }
                    case 3, 4 -> r.add("s", "e" + rnd.nextInt(4));
                    case 5 -> r.remove("s", "e" + rnd.nextInt(4));
                    case 6 -> r.write("r", "v" + rnd.nextInt(100), ts);
                    default -> inFlight.add(r.snapshot());
                }
                if (!inFlight.isEmpty() && rnd.nextInt(3) == 0) {
                    String s = inFlight.get(rnd.nextInt(inFlight.size()));
                    rs.get(rnd.nextInt(3)).merge(s); // old, duplicated, reordered deliveries
                }
            }
            sync(rs.get(0), rs.get(1));
            sync(rs.get(1), rs.get(2));
            sync(rs.get(0), rs.get(2));

            for (Replica r : rs) {
                assertThat(r.counter("c")).as("seed %d counter", seed).isEqualTo(expected[0]);
                assertThat(r.elements("s")).as("seed %d set", seed).isEqualTo(rs.get(0).elements("s"));
                assertThat(r.read("r")).as("seed %d register", seed).isEqualTo(rs.get(0).read("r"));
            }
        }
    }
}

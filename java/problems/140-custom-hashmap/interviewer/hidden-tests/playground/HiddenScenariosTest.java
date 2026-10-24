package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.KeyHasher;
import contract.StringMap;
import java.time.Duration;
import java.util.ArrayList;
import java.util.Collections;
import java.util.HashSet;
import java.util.List;
import java.util.Optional;
import java.util.Random;
import java.util.Set;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicLong;
import org.junit.jupiter.api.Assertions;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    static final KeyHasher GOOD = key -> {
        int h = key.hashCode();
        h ^= h >>> 16;
        h *= 0x85ebca6b;
        h ^= h >>> 13;
        h *= 0xc2b2ae35;
        h ^= h >>> 16;
        return h;
    };

    static final KeyHasher SAME = key -> -7;

    private static StringMap map(int cap, int lf, KeyHasher h) {
        return Candidate.of(() -> Solution.create(cap, lf, h));
    }

    @Test
    void shouldRemoveTwoHundredThousandKeysInRandomOrderWithoutScanningTheWholeMap() {
        Assertions.assertTimeoutPreemptively(Duration.ofSeconds(30), () -> {
            StringMap m = map(16, 75, GOOD);
            int n = 200_000;
            List<Integer> order = new ArrayList<>();
            for (int i = 0; i < n; i++) {
                m.put("key-" + i, "v");
                order.add(i);
            }
            Collections.shuffle(order, new Random(7));
            for (int i : order) {
                assertThat(m.remove("key-" + i)).isPresent();
            }
            assertThat(m.size()).isZero();
            assertThat(m.keys()).isEmpty();
            assertThat(m.capacity()).isEqualTo(524_288);
        });
    }

    @Test
    void shouldNotGrowWhenTheSameFewKeysAreAddedAndRemovedOverAndOver() {
        StringMap m = map(16, 75, GOOD);
        for (int round = 0; round < 100_000; round++) {
            String k = "k" + (round % 5);
            m.put(k, "v" + round);
            if (round % 5 == 4) {
                for (int j = 0; j < 5; j++) {
                    assertThat(m.remove("k" + j)).isPresent();
                }
            }
        }
        assertThat(m.capacity()).isEqualTo(16);
        assertThat(m.isEmpty()).isTrue();
        m.put("b", "1");
        m.put("a", "2");
        assertThat(m.keys()).containsExactly("b", "a");
    }

    @Test
    void shouldKeepOrderAndGrowthRuleWhenEverythingCollidesAndHalfTheKeysAreRemoved() {
        StringMap m = map(1, 50, SAME);
        for (int i = 0; i < 3_000; i++) {
            m.put("k" + i, "v" + i);
        }
        assertThat(m.capacity()).isEqualTo(8192);
        for (int i = 0; i < 3_000; i += 2) {
            m.remove("k" + i);
        }
        List<String> expected = new ArrayList<>();
        for (int i = 1; i < 3_000; i += 2) {
            expected.add("k" + i);
        }
        assertThat(m.keys()).containsExactlyElementsOf(expected);
        assertThat(m.entries().get(0)).isEqualTo("k1=v1");
        assertThat(m.capacity()).isEqualTo(8192);
    }

    @Test
    void shouldLetExactlyOneThreadWinPutIfAbsentForEachKey() {
        StringMap m = map(16, 75, GOOD);
        int keys = 2_000;
        AtomicInteger winners = new AtomicInteger();

        Concurrently.run(8, t -> {
            for (int i = 0; i < keys; i++) {
                if (m.putIfAbsent("key-" + i, "t" + t).isEmpty()) {
                    winners.incrementAndGet();
                }
            }
        });

        assertThat(winners.get()).isEqualTo(keys);
        assertThat(m.size()).isEqualTo(keys);
        for (int i = 0; i < keys; i++) {
            assertThat(m.get("key-" + i)).isPresent();
        }
    }

    @Test
    void shouldStayInternallyConsistentUnderConcurrentPutRemoveAndGetOnOverlappingKeys() {
        StringMap m = map(2, 75, GOOD);

        Concurrently.run(8, t -> {
            Random r = new Random(t);
            for (int i = 0; i < 40_000; i++) {
                String k = "k" + r.nextInt(300);
                switch (r.nextInt(4)) {
                    case 0 -> m.put(k, "t" + t);
                    case 1 -> m.remove(k);
                    case 2 -> m.get(k);
                    default -> m.putIfAbsent(k, "t" + t);
                }
            }
        });

        List<String> keys = m.keys();
        assertThat(keys).hasSize(m.size());
        assertThat(new HashSet<>(keys)).hasSize(keys.size());
        assertThat(m.entries()).hasSize(keys.size());
        for (String k : keys) {
            assertThat(m.containsKey(k)).isTrue();
        }
        for (int i = 0; i < 300; i++) {
            assertThat(m.containsKey("k" + i)).isEqualTo(keys.contains("k" + i));
        }
        assertThat(m.capacity()).isLessThanOrEqualTo(1024);
    }

    @Test
    void shouldCallTheHasherExactlyOncePerOperationEvenUnderContention() {
        AtomicLong calls = new AtomicLong();
        KeyHasher counting = key -> {
            calls.incrementAndGet();
            return GOOD.hash(key);
        };
        StringMap m = map(2, 75, counting);
        AtomicLong ops = new AtomicLong();

        Concurrently.run(8, t -> {
            for (int i = 0; i < 5_000; i++) {
                String k = "k" + ((i * 31 + t) % 700);
                m.put(k, "v");
                m.get(k);
                m.remove(k);
                m.size();
                m.keys();
                ops.addAndGet(3);
            }
        });

        assertThat(calls.get()).isEqualTo(ops.get());
    }

    @Test
    void shouldReturnAtomicSnapshotsOfKeysWhileOtherThreadsInsert() {
        StringMap m = map(2, 75, GOOD);
        int perThread = 3_000;
        List<List<String>> snapshots = Collections.synchronizedList(new ArrayList<>());

        Concurrently.run(6, t -> {
            if (t >= 4) {
                for (int i = 0; i < 100; i++) {
                    snapshots.add(m.keys());
                    assertThat(m.entries().size()).isLessThanOrEqualTo(4 * perThread);
                }
                return;
            }
            for (int i = 0; i < perThread; i++) {
                m.put("t" + t + "-" + String.format("%05d", i), "v");
            }
        });

        for (List<String> snap : snapshots) {
            assertThat(new HashSet<>(snap)).hasSize(snap.size());
            int[] last = {-1, -1, -1, -1};
            for (String k : snap) {
                int t = k.charAt(1) - '0';
                int i = Integer.parseInt(k.substring(3));
                assertThat(i).as("per-thread insertion order inside a snapshot").isGreaterThan(last[t]);
                last[t] = i;
            }
        }
        assertThat(m.size()).isEqualTo(4 * perThread);
    }

    @Test
    void shouldBeUsableAsAGeneralPurposeMapAfterClearAndReuse() {
        StringMap m = map(3, 40, GOOD);
        for (int round = 0; round < 5; round++) {
            for (int i = 0; i < 100; i++) {
                m.put("r" + round + "-" + i, "v");
            }
            assertThat(m.size()).isEqualTo(100);
            assertThat(m.keys().get(0)).isEqualTo("r" + round + "-0");
            m.clear();
            assertThat(m.get("r" + round + "-0")).isEmpty();
        }
        assertThat(m.capacity()).isEqualTo(256);
        Optional<String> none = m.put("again", "1");
        assertThat(none).isEmpty();
    }
}

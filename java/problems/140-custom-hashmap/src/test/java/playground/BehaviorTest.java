package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.KeyHasher;
import contract.StringMap;
import java.time.Duration;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Optional;
import java.util.Set;
import java.util.concurrent.atomic.AtomicLong;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Assertions;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    /** A well-mixed hash (murmur3 finaliser over String.hashCode()). */
    static final KeyHasher GOOD = key -> {
        int h = key.hashCode();
        h ^= h >>> 16;
        h *= 0x85ebca6b;
        h ^= h >>> 13;
        h *= 0xc2b2ae35;
        h ^= h >>> 16;
        return h;
    };

    /** Everything collides. */
    static final KeyHasher SAME = key -> 42;

    /** Always negative, sometimes Integer.MIN_VALUE. */
    static final KeyHasher NEGATIVE = key -> key.length() % 3 == 0 ? Integer.MIN_VALUE : -1 - (key.hashCode() & 0x7fffffff);

    /** Extreme values only. */
    static final KeyHasher EXTREME = key -> key.length() % 2 == 0 ? Integer.MAX_VALUE : Integer.MIN_VALUE;

    static final class Counting implements KeyHasher {
        final AtomicLong calls = new AtomicLong();
        private final KeyHasher inner;

        Counting(KeyHasher inner) {
            this.inner = inner;
        }

        @Override
        public int hash(String key) {
            calls.incrementAndGet();
            return inner.hash(key);
        }
    }

    private static StringMap map() {
        return Candidate.of(() -> Solution.create(16, 75, GOOD));
    }

    private static StringMap map(int cap, int lf, KeyHasher h) {
        return Candidate.of(() -> Solution.create(cap, lf, h));
    }

    /** The growth rule from the README, restated as a model. */
    private static int modelCapacity(int initial, int lf, int inserts) {
        int cap = 1;
        while (cap < initial) {
            cap <<= 1;
        }
        for (int size = 1; size <= inserts; size++) {
            if ((long) size * 100 > (long) cap * lf) {
                cap *= 2;
            }
        }
        return cap;
    }

    @Test
    void shouldStoreAndReturnValuesAndPreviousValues() {
        StringMap m = map();

        assertThat(m.put("a", "1")).isEmpty();
        assertThat(m.put("b", "2")).isEmpty();
        assertThat(m.get("a")).contains("1");
        assertThat(m.put("a", "one")).contains("1");
        assertThat(m.get("a")).contains("one");
        assertThat(m.get("missing")).isEmpty();
        assertThat(m.containsKey("a")).isTrue();
        assertThat(m.containsKey("zzz")).isFalse();
        assertThat(m.size()).isEqualTo(2);
    }

    @Test
    void shouldTreatKeysAsCaseSensitiveAndAllowTheEmptyStringAsKeyAndValue() {
        StringMap m = map();
        m.put("Key", "upper");
        m.put("key", "lower");
        m.put("", "empty-key");
        m.put("blank-value", "");

        assertThat(m.size()).isEqualTo(4);
        assertThat(m.get("Key")).contains("upper");
        assertThat(m.get("key")).contains("lower");
        assertThat(m.get("")).contains("empty-key");
        assertThat(m.get("blank-value")).contains("");
        assertThat(m.containsKey("")).isTrue();
        assertThat(m.remove("")).contains("empty-key");
        assertThat(m.containsKey("")).isFalse();
    }

    @Test
    void shouldRejectNullKeysAndValuesWithoutChangingTheMap() {
        StringMap m = map();
        m.put("a", "1");

        assertThatThrownBy(() -> m.put(null, "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.put("k", null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.putIfAbsent(null, "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.putIfAbsent("k", null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.get(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.getOrDefault(null, "d")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.getOrDefault("a", null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.remove(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.containsKey(null)).isInstanceOf(IllegalArgumentException.class);

        assertThat(m.size()).isEqualTo(1);
        assertThat(m.keys()).containsExactly("a");
        assertThat(m.containsKey("k")).isFalse();
    }

    @Test
    void shouldRejectInvalidCreationParameters() {
        map(); // obtains a valid instance first (skips while unimplemented)

        assertThatThrownBy(() -> Solution.create(0, 75, GOOD)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(-5, 75, GOOD)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(1_048_577, 75, GOOD)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(16, 9, GOOD)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(16, 101, GOOD)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(16, 75, null)).isInstanceOf(IllegalArgumentException.class);
        assertThat(Solution.create(1, 10, GOOD).capacity()).isEqualTo(1);
        assertThat(Solution.create(1_048_576, 100, GOOD).capacity()).isEqualTo(1_048_576);
    }

    @Test
    void shouldRemoveAndReportThePreviousValue() {
        StringMap m = map();
        m.put("a", "1");
        m.put("b", "2");

        assertThat(m.remove("a")).contains("1");
        assertThat(m.remove("a")).isEmpty();
        assertThat(m.remove("never")).isEmpty();
        assertThat(m.get("a")).isEmpty();
        assertThat(m.get("b")).contains("2");
        assertThat(m.size()).isEqualTo(1);
    }

    @Test
    void shouldReportSizeAndEmptinessAndClearWithoutShrinking() {
        StringMap m = map(2, 75, GOOD);
        assertThat(m.isEmpty()).isTrue();
        assertThat(m.size()).isZero();

        for (int i = 0; i < 50; i++) {
            m.put("k" + i, "v");
        }
        int cap = m.capacity();
        assertThat(m.isEmpty()).isFalse();
        assertThat(m.size()).isEqualTo(50);

        m.clear();
        assertThat(m.isEmpty()).isTrue();
        assertThat(m.size()).isZero();
        assertThat(m.keys()).isEmpty();
        assertThat(m.get("k1")).isEmpty();
        assertThat(m.capacity()).isEqualTo(cap);

        m.put("z", "1");
        m.put("a", "2");
        assertThat(m.keys()).containsExactly("z", "a");
    }

    @Test
    void shouldStartAtTheSmallestPowerOfTwoAtLeastTheInitialCapacity() {
        int[][] cases = {{1, 1}, {2, 2}, {3, 4}, {5, 8}, {16, 16}, {17, 32}, {100, 128}, {1000, 1024}, {1024, 1024}};
        for (int[] c : cases) {
            assertThat(map(c[0], 75, GOOD).capacity()).as("initial %d", c[0]).isEqualTo(c[1]);
        }
    }

    @Test
    void shouldFollowTheDocumentedCapacityTableForDefaultParameters() {
        StringMap m = map(16, 75, GOOD);
        int[][] checkpoints = {{1, 16}, {12, 16}, {13, 32}, {24, 32}, {25, 64}, {48, 64}, {49, 128}, {96, 128}, {97, 256}};
        int inserted = 0;
        for (int[] cp : checkpoints) {
            while (inserted < cp[0]) {
                m.put("key-" + inserted, "v");
                inserted++;
            }
            assertThat(m.capacity()).as("after %d inserts", cp[0]).isEqualTo(cp[1]);
        }
    }

    @Test
    void shouldDoubleAtMostOncePerInsertionAndFollowTheRuleForOtherLoadFactors() {
        // capacity 1, load factor 10%: every one of the first 7 insertions is over the limit and doubles once
        StringMap m = map(1, 10, GOOD);
        int[] expected = {2, 4, 8, 16, 32, 64, 128, 128, 128};
        for (int i = 0; i < expected.length; i++) {
            m.put("k" + i, "v");
            assertThat(m.capacity()).as("after %d inserts", i + 1).isEqualTo(expected[i]);
        }

        // load factor 100%: grows when size exceeds capacity
        StringMap full = map(4, 100, GOOD);
        for (int i = 0; i < 4; i++) {
            full.put("k" + i, "v");
        }
        assertThat(full.capacity()).isEqualTo(4);
        full.put("k4", "v");
        assertThat(full.capacity()).isEqualTo(8);

        for (int lf : new int[] {10, 33, 50, 75, 90, 100}) {
            StringMap x = map(7, lf, GOOD);
            for (int i = 1; i <= 600; i++) {
                x.put("n" + i, "v");
                if (i % 50 == 0 || i < 20) {
                    assertThat(x.capacity()).as("lf %d after %d inserts", lf, i).isEqualTo(modelCapacity(7, lf, i));
                }
            }
        }
    }

    @Test
    void shouldGrowOnlyWhenANewKeyIsInsertedAndNeverShrink() {
        StringMap m = map(4, 100, GOOD);
        for (int i = 0; i < 4; i++) {
            m.put("k" + i, "v0");
        }
        for (int round = 0; round < 10; round++) {
            for (int i = 0; i < 4; i++) {
                m.put("k" + i, "v" + round);
                m.putIfAbsent("k" + i, "ignored");
            }
        }
        assertThat(m.capacity()).isEqualTo(4);

        m.put("k4", "v");
        assertThat(m.capacity()).isEqualTo(8);

        for (int i = 0; i < 5; i++) {
            m.remove("k" + i);
        }
        assertThat(m.size()).isZero();
        assertThat(m.capacity()).isEqualTo(8);

        // removing and re-adding the same keys: growth depends on size only
        for (int i = 0; i < 8; i++) {
            m.put("k" + i, "v");
        }
        assertThat(m.capacity()).isEqualTo(8);
        m.put("k8", "v");
        assertThat(m.capacity()).isEqualTo(16);
    }

    @Test
    void shouldListKeysInInsertionOrderAndKeepThemStableAcrossResizes() {
        StringMap m = map(1, 75, GOOD);
        List<String> expected = new ArrayList<>();
        for (int i = 0; i < 300; i++) {
            String k = "key-" + ((i * 7919) % 1000) + "-" + i;
            m.put(k, "v");
            expected.add(k);
        }
        assertThat(m.capacity()).isGreaterThan(300 * 100 / 75 / 2);
        assertThat(m.keys()).containsExactlyElementsOf(expected);
    }

    @Test
    void shouldKeepThePositionOnReputAndMoveToTheEndOnRemoveThenPut() {
        StringMap m = map();
        m.put("a", "1");
        m.put("b", "2");
        m.put("c", "3");

        m.put("a", "again");
        assertThat(m.keys()).containsExactly("a", "b", "c");

        m.putIfAbsent("b", "ignored");
        assertThat(m.keys()).containsExactly("a", "b", "c");

        m.remove("a");
        m.put("a", "new");
        assertThat(m.keys()).containsExactly("b", "c", "a");

        m.remove("c");
        assertThat(m.keys()).containsExactly("b", "a");
        m.put("d", "4");
        m.put("c", "5");
        assertThat(m.keys()).containsExactly("b", "a", "d", "c");
    }

    @Test
    void shouldListEntriesAsKeyEqualsValueInTheSameOrderAsKeys() {
        StringMap m = map();
        m.put("x", "1");
        m.put("", "empty");
        m.put("k=v", "a=b");
        m.put("blank", "");
        m.put("x", "2");

        assertThat(m.entries()).containsExactly("x=2", "=empty", "k=v=a=b", "blank=");
        assertThat(m.keys()).containsExactly("x", "", "k=v", "blank");
    }

    @Test
    void shouldReturnSnapshotsFromKeysAndEntries() {
        StringMap m = map();
        m.put("a", "1");
        List<String> keys = m.keys();
        List<String> entries = m.entries();

        m.put("b", "2");
        m.remove("a");

        assertThat(keys).containsExactly("a");
        assertThat(entries).containsExactly("a=1");
        assertThat(m.keys()).containsExactly("b");
    }

    @Test
    void shouldPutIfAbsentOnlyWhenTheKeyIsMissing() {
        StringMap m = map();

        assertThat(m.putIfAbsent("a", "1")).isEmpty();
        assertThat(m.putIfAbsent("a", "2")).contains("1");
        assertThat(m.get("a")).contains("1");
        assertThat(m.size()).isEqualTo(1);
        m.remove("a");
        assertThat(m.putIfAbsent("a", "3")).isEmpty();
        assertThat(m.get("a")).contains("3");
    }

    @Test
    void shouldReturnTheDefaultFromGetOrDefaultWithoutStoringIt() {
        StringMap m = map();
        m.put("a", "1");

        assertThat(m.getOrDefault("a", "d")).isEqualTo("1");
        assertThat(m.getOrDefault("b", "d")).isEqualTo("d");
        assertThat(m.getOrDefault("b", "")).isEqualTo("");
        assertThat(m.containsKey("b")).isFalse();
        assertThat(m.size()).isEqualTo(1);
    }

    @Test
    void shouldCallTheSuppliedHasherExactlyOncePerKeyedOperationAndNeverForTheOthers() {
        Counting h = new Counting(GOOD);
        StringMap m = map(2, 50, h);
        long before;

        before = h.calls.get();
        m.put("a", "1");
        assertThat(h.calls.get() - before).as("put new").isEqualTo(1);
        before = h.calls.get();
        m.put("a", "2");
        assertThat(h.calls.get() - before).as("put existing").isEqualTo(1);
        before = h.calls.get();
        m.putIfAbsent("b", "1");
        m.putIfAbsent("b", "1");
        assertThat(h.calls.get() - before).as("two putIfAbsent").isEqualTo(2);
        before = h.calls.get();
        m.get("a");
        m.get("nope");
        m.getOrDefault("a", "d");
        m.getOrDefault("nope", "d");
        m.containsKey("a");
        m.containsKey("nope");
        assertThat(h.calls.get() - before).as("six reads").isEqualTo(6);
        before = h.calls.get();
        m.remove("a");
        m.remove("nope");
        assertThat(h.calls.get() - before).as("two removes").isEqualTo(2);

        before = h.calls.get();
        m.size();
        m.isEmpty();
        m.capacity();
        m.keys();
        m.entries();
        m.clear();
        assertThat(h.calls.get() - before).as("size/isEmpty/capacity/keys/entries/clear").isZero();

        // growing must not ask the hasher again for the keys that are already stored
        for (int i = 0; i < 200; i++) {
            before = h.calls.get();
            m.put("grow-" + i, "v");
            assertThat(h.calls.get() - before).as("put #%d (may resize)", i).isEqualTo(1);
        }
        assertThat(m.capacity()).isGreaterThan(200);
    }

    @Test
    void shouldNotCallTheHasherForRejectedArguments() {
        Counting h = new Counting(GOOD);
        StringMap m = map(16, 75, h);

        assertThatThrownBy(() -> m.put(null, "v")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.put("k", null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> m.get(null)).isInstanceOf(IllegalArgumentException.class);
        assertThat(h.calls.get()).isZero();
    }

    @Test
    void shouldStayCorrectWithHashersReturningNegativeAndExtremeValues() {
        for (KeyHasher h : new KeyHasher[] {NEGATIVE, EXTREME}) {
            StringMap m = map(4, 75, h);
            for (int i = 0; i < 2_000; i++) {
                assertThat(m.put("key-" + i, "v" + i)).isEmpty();
            }
            assertThat(m.size()).isEqualTo(2_000);
            for (int i = 0; i < 2_000; i++) {
                assertThat(m.get("key-" + i)).contains("v" + i);
            }
            for (int i = 0; i < 2_000; i += 2) {
                assertThat(m.remove("key-" + i)).contains("v" + i);
            }
            assertThat(m.size()).isEqualTo(1_000);
            assertThat(m.containsKey("key-0")).isFalse();
            assertThat(m.containsKey("key-1")).isTrue();
            assertThat(m.capacity()).isEqualTo(modelCapacity(4, 75, 2_000));
        }
    }

    @Test
    void shouldKeepCollidingKeysApartWhenEveryKeyHasTheSameHash() {
        StringMap m = map(2, 75, SAME);
        m.put("Aa", "1"); // "Aa" and "BB" even share String.hashCode()
        m.put("BB", "2");
        m.put("c", "3");

        assertThat(m.get("Aa")).contains("1");
        assertThat(m.get("BB")).contains("2");
        assertThat(m.remove("BB")).contains("2");
        assertThat(m.get("Aa")).contains("1");
        assertThat(m.get("c")).contains("3");
        assertThat(m.containsKey("BB")).isFalse();
        assertThat(m.put("Aa", "9")).contains("1");
        assertThat(m.keys()).containsExactly("Aa", "c");
        assertThat(m.capacity()).isEqualTo(modelCapacity(2, 75, 3));
    }

    @Test
    void shouldHandleFiveThousandKeysThatAllCollideWithinTheDocumentedBound() {
        Assertions.assertTimeoutPreemptively(Duration.ofSeconds(20), () -> {
            StringMap m = map(16, 75, SAME);
            for (int i = 0; i < 5_000; i++) {
                assertThat(m.put("key-" + i, "v" + i)).isEmpty();
            }
            assertThat(m.size()).isEqualTo(5_000);
            for (int i = 0; i < 5_000; i++) {
                assertThat(m.get("key-" + i)).contains("v" + i);
            }
            for (int i = 0; i < 5_000; i += 2) {
                assertThat(m.remove("key-" + i)).contains("v" + i);
            }
            assertThat(m.size()).isEqualTo(2_500);
            assertThat(m.containsKey("key-2")).isFalse();
            assertThat(m.get("key-4999")).contains("v4999");
            assertThat(m.capacity()).isEqualTo(modelCapacity(16, 75, 5_000));
            assertThat(m.keys().get(0)).isEqualTo("key-1");
        });
    }

    @Test
    void shouldHandleOneMillionPutsAndGetsWithAGoodHasherWithinTheDocumentedBound() {
        Assertions.assertTimeoutPreemptively(Duration.ofSeconds(60), () -> {
            StringMap m = map(16, 75, GOOD);
            int n = 1_000_000;
            for (int i = 0; i < n; i++) {
                m.put("key-" + i, "v" + i);
            }
            assertThat(m.size()).isEqualTo(n);
            for (int i = 0; i < n; i++) {
                assertThat(m.get("key-" + i)).contains("v" + i);
            }
            assertThat(m.get("key-" + n)).isEmpty();
            assertThat(m.capacity()).isEqualTo(2_097_152);
            assertThat(m.capacity()).isEqualTo(modelCapacity(16, 75, n));
        });
    }

    @Test
    void shouldLoseNothingWhenEightThreadsPutDisjointKeys() {
        StringMap m = map(16, 75, GOOD);
        int perThread = 5_000;

        Concurrently.run(8, t -> {
            for (int i = 0; i < perThread; i++) {
                m.put("t" + t + "-" + i, "v" + i);
            }
        });

        assertThat(m.size()).isEqualTo(8 * perThread);
        assertThat(m.capacity()).isEqualTo(modelCapacity(16, 75, 8 * perThread));
        for (int t = 0; t < 8; t++) {
            for (int i = 0; i < perThread; i += 97) {
                assertThat(m.get("t" + t + "-" + i)).contains("v" + i);
            }
        }
        Set<String> distinct = new HashSet<>(m.keys());
        assertThat(distinct).hasSize(8 * perThread);
    }
}

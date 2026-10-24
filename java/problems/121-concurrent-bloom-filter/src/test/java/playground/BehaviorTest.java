package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.ConcurrentBloomFilter;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicLong;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private static ConcurrentBloomFilter filter(int expectedItems, double rate) {
        return Candidate.of(() -> Solution.create(expectedItems, rate));
    }

    /** Very low false positive rate: for a handful of items, "might contain" is effectively exact. */
    private static ConcurrentBloomFilter roomy() {
        return filter(1_000, 1e-9);
    }

    private static List<String> items(String prefix, int from, int to) {
        List<String> out = new ArrayList<>();
        for (int i = from; i < to; i++) {
            out.add(prefix + i);
        }
        return out;
    }

    private static long expectedSize(long m, int k, long x) {
        if (x == 0) {
            return 0;
        }
        return Math.round(-((double) m / k) * Math.log(1.0 - (double) x / m));
    }

    // ---------- sizing ----------

    @Test
    void shouldSizeAThousandItemsAtOnePercent() {
        ConcurrentBloomFilter f = filter(1_000, 0.01);

        assertThat(f.bitCount()).isEqualTo(9_586);
        assertThat(f.hashCount()).isEqualTo(7);
    }

    @Test
    void shouldSizeOtherParameterSetsWithTheDocumentedFormulas() {
        assertThat(filter(10_000, 0.01).bitCount()).isEqualTo(95_851);
        assertThat(filter(10_000, 0.01).hashCount()).isEqualTo(7);
        assertThat(filter(100, 0.001).bitCount()).isEqualTo(1_438);
        assertThat(filter(100, 0.001).hashCount()).isEqualTo(10);
        assertThat(filter(500, 0.05).bitCount()).isEqualTo(3_118);
        assertThat(filter(500, 0.05).hashCount()).isEqualTo(4);
        assertThat(filter(100, 1e-9).bitCount()).isEqualTo(4_314);
        assertThat(filter(100, 1e-9).hashCount()).isEqualTo(30);
        assertThat(filter(1_000_000, 0.001).bitCount()).isEqualTo(14_377_588);
        assertThat(filter(1_000_000, 0.001).hashCount()).isEqualTo(10);
    }

    @Test
    void shouldUseAtLeastOneHashEvenForAVeryLooseRate() {
        ConcurrentBloomFilter f = filter(10, 0.9);

        assertThat(f.bitCount()).isEqualTo(3);
        assertThat(f.hashCount()).isEqualTo(1);
    }

    @Test
    void shouldRoundTheHashCountToTheNearestInteger() {
        // m/n*ln2 = 1.386 -> 1 ; 3.327 -> 3 ; 4.322 -> 4 ; 6.644 -> 7
        assertThat(filter(1, 0.5).hashCount()).isEqualTo(1);
        assertThat(filter(50, 0.1).hashCount()).isEqualTo(3);
        assertThat(filter(500, 0.05).hashCount()).isEqualTo(4);
        assertThat(filter(1_000, 0.01).hashCount()).isEqualTo(7);
    }

    @Test
    void shouldRejectInvalidExpectedItems() {
        filter(10, 0.01); // proves the factory is wired

        assertThatThrownBy(() -> Solution.create(0, 0.01)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(-5, 0.01)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRejectInvalidFalsePositiveRates() {
        filter(10, 0.01); // proves the factory is wired

        for (double bad : new double[] {0.0, -0.1, 1.0, 1.5, Double.NaN, Double.POSITIVE_INFINITY}) {
            assertThatThrownBy(() -> Solution.create(10, bad))
                    .as("rate %s", bad)
                    .isInstanceOf(IllegalArgumentException.class);
        }
    }

    // ---------- add / mightContain ----------

    @Test
    void shouldStartEmpty() {
        ConcurrentBloomFilter f = roomy();

        assertThat(f.setBits()).isZero();
        assertThat(f.approximateSize()).isZero();
        assertThat(f.mightContain("anything")).isFalse();
    }

    @Test
    void shouldReportANewItemAsChangedAndPresent() {
        ConcurrentBloomFilter f = roomy();

        assertThat(f.add("apple")).isTrue();
        assertThat(f.mightContain("apple")).isTrue();
    }

    @Test
    void shouldReportAnAlreadyAddedItemAsUnchanged() {
        ConcurrentBloomFilter f = roomy();
        f.add("apple");
        long bits = f.setBits();

        assertThat(f.add("apple")).isFalse();
        assertThat(f.add("apple")).isFalse();
        assertThat(f.setBits()).isEqualTo(bits);
        assertThat(f.mightContain("apple")).isTrue();
    }

    @Test
    void shouldSetBetweenOneAndKBitsForASingleItem() {
        ConcurrentBloomFilter f = filter(1_000, 0.01);

        f.add("apple");

        assertThat(f.setBits()).isBetween(1L, (long) f.hashCount());
    }

    @Test
    void shouldNotChangeTheFilterWhenProbing() {
        ConcurrentBloomFilter f = roomy();
        f.add("apple");
        long bits = f.setBits();
        long size = f.approximateSize();

        for (int i = 0; i < 200; i++) {
            f.mightContain("probe-" + i);
        }

        assertThat(f.setBits()).isEqualTo(bits);
        assertThat(f.approximateSize()).isEqualTo(size);
        assertThat(f.add("probe-0")).isTrue(); // probing did not add it
    }

    @Test
    void shouldTreatItemsAsCaseSensitiveAndUntrimmed() {
        ConcurrentBloomFilter f = roomy();
        f.add("Apple");

        assertThat(f.mightContain("Apple")).isTrue();
        assertThat(f.mightContain("apple")).isFalse();
        assertThat(f.mightContain("APPLE")).isFalse();
        assertThat(f.add(" Apple ")).isTrue();
        assertThat(f.mightContain(" Apple ")).isTrue();
        assertThat(f.add("apple")).isTrue();
    }

    @Test
    void shouldHandleNonAsciiItems() {
        ConcurrentBloomFilter f = roomy();

        assertThat(f.add("école")).isTrue();
        assertThat(f.add("日本語")).isTrue();
        assertThat(f.add("😀 smile")).isTrue();
        assertThat(f.mightContain("école")).isTrue();
        assertThat(f.mightContain("日本語")).isTrue();
        assertThat(f.mightContain("😀 smile")).isTrue();
        assertThat(f.mightContain("Ecole")).isFalse();
        assertThat(f.mightContain("ecole")).isFalse();
    }

    @Test
    void shouldRejectNullAndBlankItems() {
        ConcurrentBloomFilter f = roomy();

        for (String bad : new String[] {null, "", "   ", "\t\n "}) {
            assertThatThrownBy(() -> f.add(bad)).as("add %s", bad).isInstanceOf(IllegalArgumentException.class);
            assertThatThrownBy(() -> f.mightContain(bad))
                    .as("mightContain %s", bad)
                    .isInstanceOf(IllegalArgumentException.class);
        }
    }

    @Test
    void shouldChangeNothingWhenAnItemIsRejected() {
        ConcurrentBloomFilter f = roomy();
        f.add("apple");
        long bits = f.setBits();

        assertThatThrownBy(() -> f.add("  ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.add(null)).isInstanceOf(IllegalArgumentException.class);

        assertThat(f.setBits()).isEqualTo(bits);
        assertThat(f.approximateSize()).isEqualTo(1);
    }

    @Test
    void shouldReportEveryNewItemAsChangedInARoomyFilter() {
        ConcurrentBloomFilter f = roomy();

        int changed = 0;
        for (String s : items("n-", 0, 300)) {
            if (f.add(s)) {
                changed++;
            }
        }

        assertThat(changed).isEqualTo(300);
        assertThat(f.add("n-0")).isFalse();
        assertThat(f.add("n-299")).isFalse();
    }

    @Test
    void shouldKeepTwoFiltersIndependent() {
        ConcurrentBloomFilter a = roomy();
        ConcurrentBloomFilter b = roomy();

        a.add("apple");

        assertThat(a.mightContain("apple")).isTrue();
        assertThat(b.mightContain("apple")).isFalse();
        assertThat(b.setBits()).isZero();
    }

    // ---------- addAll ----------

    @Test
    void shouldCountTheItemsThatChangedTheFilterInAddAll() {
        ConcurrentBloomFilter f = roomy();
        f.add("a");

        int changed = f.addAll(List.of("a", "b", "c", "b", "d"));

        assertThat(changed).isEqualTo(3); // b, c, d; the repeated b and the earlier a changed nothing
        for (String s : List.of("a", "b", "c", "d")) {
            assertThat(f.mightContain(s)).isTrue();
        }
    }

    @Test
    void shouldAddNothingWhenAnyElementOfAddAllIsInvalid() {
        ConcurrentBloomFilter f = roomy();
        List<String> batch = new ArrayList<>(List.of("a", "b"));
        batch.add(" ");
        List<String> withNull = new ArrayList<>(List.of("a", "b"));
        withNull.add(null);

        assertThatThrownBy(() -> f.addAll(batch)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.addAll(withNull)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> f.addAll(null)).isInstanceOf(IllegalArgumentException.class);

        assertThat(f.setBits()).isZero();
        assertThat(f.mightContain("a")).isFalse();
    }

    @Test
    void shouldAcceptAnEmptyAddAll() {
        ConcurrentBloomFilter f = roomy();

        assertThat(f.addAll(Collections.emptyList())).isZero();
        assertThat(f.setBits()).isZero();
    }

    // ---------- approximateSize ----------

    @Test
    void shouldEstimateTheSizeFromTheNumberOfSetBits() {
        ConcurrentBloomFilter f = filter(1_000, 0.01);
        for (int step = 0; step < 5; step++) {
            f.addAll(items("e-" + step + "-", 0, 100));

            long expected = expectedSize(f.bitCount(), f.hashCount(), f.setBits());
            assertThat(f.approximateSize()).isBetween(expected - 1, expected + 1);
        }
    }

    @Test
    void shouldEstimateTheNumberOfDistinctItems() {
        ConcurrentBloomFilter f = filter(1_000, 0.01);
        f.addAll(items("d-", 0, 1_000));
        f.addAll(items("d-", 0, 1_000)); // duplicates do not count

        assertThat(f.approximateSize()).isBetween(950L, 1_050L);
    }

    @Test
    void shouldEstimateOneForASingleItem() {
        ConcurrentBloomFilter f = filter(1_000, 0.01);

        f.add("only");

        assertThat(f.approximateSize()).isEqualTo(1);
    }

    @Test
    void shouldNeverDecreaseTheEstimateWhileAdding() {
        ConcurrentBloomFilter f = filter(500, 0.01);
        long previous = 0;
        for (int i = 0; i < 500; i++) {
            f.add("m-" + i);
            long now = f.approximateSize();
            assertThat(now).isGreaterThanOrEqualTo(previous);
            previous = now;
        }
    }

    // ---------- clear ----------

    @Test
    void shouldForgetEverythingOnClear() {
        ConcurrentBloomFilter f = roomy();
        f.addAll(items("c-", 0, 50));

        f.clear();

        assertThat(f.setBits()).isZero();
        assertThat(f.approximateSize()).isZero();
        for (String s : items("c-", 0, 50)) {
            assertThat(f.mightContain(s)).isFalse();
        }
        assertThat(f.bitCount()).isEqualTo(43_133);
        assertThat(f.hashCount()).isEqualTo(30);
    }

    @Test
    void shouldBeUsableAgainAfterClear() {
        ConcurrentBloomFilter f = roomy();
        f.add("apple");
        f.clear();

        assertThat(f.add("apple")).isTrue();
        assertThat(f.mightContain("apple")).isTrue();
        assertThat(f.approximateSize()).isEqualTo(1);
        f.clear();
        f.clear(); // clearing an empty filter is fine
        assertThat(f.setBits()).isZero();
    }

    // ---------- hashing ----------

    @Test
    void shouldGiveTheSameBitsRegardlessOfInsertionOrderAndInstance() {
        List<String> all = items("h-", 0, 2_000);
        List<String> shuffled = new ArrayList<>(all);
        Collections.shuffle(shuffled, new java.util.Random(42));
        ConcurrentBloomFilter a = filter(2_000, 0.01);
        ConcurrentBloomFilter b = filter(2_000, 0.01);

        a.addAll(all);
        b.addAll(shuffled);

        assertThat(b.setBits()).isEqualTo(a.setBits());
        assertThat(b.approximateSize()).isEqualTo(a.approximateSize());
        for (int i = 0; i < 5_000; i++) {
            String probe = "probe-" + i;
            assertThat(b.mightContain(probe)).as(probe).isEqualTo(a.mightContain(probe));
        }
    }

    // ---------- accuracy ----------

    @Test
    void shouldHaveNoFalseNegatives() {
        ConcurrentBloomFilter f = filter(100_000, 0.01);
        List<String> all = items("item-", 0, 100_000);

        f.addAll(all);

        for (String s : all) {
            assertThat(f.mightContain(s)).as(s).isTrue();
        }
    }

    @Test
    void shouldKeepTheFalsePositiveRateWithinThreeTimesTheConfiguredRate() {
        ConcurrentBloomFilter f = filter(10_000, 0.01);
        f.addAll(items("item-", 0, 10_000));

        int falsePositives = 0;
        for (int i = 0; i < 100_000; i++) {
            if (f.mightContain("probe-" + i)) {
                falsePositives++;
            }
        }

        assertThat(falsePositives).isLessThanOrEqualTo(3_000);
    }

    @Test
    void shouldKeepTheFalsePositiveRateForATighterRate() {
        ConcurrentBloomFilter f = filter(20_000, 0.001);
        f.addAll(items("user:", 0, 20_000));

        int falsePositives = 0;
        for (int i = 0; i < 100_000; i++) {
            if (f.mightContain("other:" + i)) {
                falsePositives++;
            }
        }

        assertThat(falsePositives).isLessThanOrEqualTo(300);
    }

    // ---------- concurrency ----------

    @Test
    void shouldHaveNoFalseNegativesWhenThreadsAddDisjointRanges() {
        int threads = 8;
        int perThread = 12_500;
        ConcurrentBloomFilter f = filter(threads * perThread, 0.01);

        Concurrently.run(threads, t -> {
            for (int i = 0; i < perThread; i++) {
                f.add("t" + t + "-" + i);
            }
        });

        for (int t = 0; t < threads; t++) {
            for (int i = 0; i < perThread; i++) {
                assertThat(f.mightContain("t" + t + "-" + i)).as("t%d-%d", t, i).isTrue();
            }
        }
    }

    @Test
    void shouldEndWithTheSameBitsAsASequentialRunAfterConcurrentAdds() {
        int threads = 8;
        int perThread = 5_000;
        ConcurrentBloomFilter concurrent = filter(threads * perThread, 0.01);
        ConcurrentBloomFilter sequential = filter(threads * perThread, 0.01);

        Concurrently.run(threads, t -> concurrent.addAll(items("t" + t + "-", 0, perThread)));
        for (int t = 0; t < threads; t++) {
            sequential.addAll(items("t" + t + "-", 0, perThread));
        }

        assertThat(concurrent.setBits()).isEqualTo(sequential.setBits());
        assertThat(concurrent.approximateSize()).isEqualTo(sequential.approximateSize());
    }

    @Test
    void shouldEndWithTheSameBitsWhenThreadsAddTheSameItems() {
        int threads = 8;
        List<String> shared = items("s-", 0, 5_000);
        ConcurrentBloomFilter concurrent = filter(5_000, 0.01);
        ConcurrentBloomFilter sequential = filter(5_000, 0.01);

        Concurrently.run(threads, t -> {
            for (String s : shared) {
                concurrent.add(s);
            }
        });
        sequential.addAll(shared);

        assertThat(concurrent.setBits()).isEqualTo(sequential.setBits());
    }

    @Test
    void shouldReportAtLeastOneChangeAndAtMostKWhenThreadsAddTheSameNewItem() {
        for (int round = 0; round < 20; round++) {
            ConcurrentBloomFilter f = roomy();
            AtomicInteger changed = new AtomicInteger();

            Concurrently.run(16, t -> {
                if (f.add("contested")) {
                    changed.incrementAndGet();
                }
            });

            assertThat(changed.get()).isBetween(1, f.hashCount());
            assertThat(f.setBits()).isBetween(1L, (long) f.hashCount());
            assertThat(f.add("contested")).isFalse();
        }
    }

    @Test
    void shouldNeverShowAnAddedItemAsAbsentOnceAddHasReturned() {
        int total = 20_000;
        ConcurrentBloomFilter f = filter(total, 0.01);
        AtomicInteger published = new AtomicInteger(); // items [0, published) have been added and add() returned
        AtomicLong missing = new AtomicLong();

        Concurrently.run(4, t -> {
            if (t == 0) {
                for (int i = 0; i < total; i++) {
                    f.add("w-" + i);
                    published.set(i + 1);
                }
            } else {
                int checked = 0;
                int bound;
                do {
                    bound = published.get();
                    for (; checked < bound; checked++) {
                        if (!f.mightContain("w-" + checked)) {
                            missing.incrementAndGet();
                        }
                    }
                    // re-check the newest published item right away, it is the most likely to be racing
                    if (bound > 0 && !f.mightContain("w-" + (bound - 1))) {
                        missing.incrementAndGet();
                    }
                } while (bound < total);
            }
        });

        assertThat(missing.get()).isZero();
    }

    @Test
    void shouldAnswerProbesWhileOtherThreadsAdd() {
        ConcurrentBloomFilter f = filter(40_000, 0.01);
        f.addAll(items("base-", 0, 10_000));
        AtomicLong missing = new AtomicLong();

        Concurrently.run(8, t -> {
            if (t < 4) {
                for (int i = 0; i < 7_500; i++) {
                    f.add("late-" + t + "-" + i);
                }
            } else {
                for (int round = 0; round < 3; round++) {
                    for (int i = 0; i < 10_000; i++) {
                        if (!f.mightContain("base-" + i)) {
                            missing.incrementAndGet();
                        }
                    }
                }
            }
        });

        assertThat(missing.get()).isZero();
        assertThat(f.mightContain("late-3-7499")).isTrue();
    }
}

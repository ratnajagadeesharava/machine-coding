package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.ParallelSorter;
import java.time.Duration;
import java.util.Arrays;
import java.util.Random;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import org.junit.jupiter.api.Assertions;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private ParallelSorter sorter() {
        return Candidate.of(Solution::create);
    }

    private static int[] random(int n, long seed) {
        Random r = new Random(seed);
        int[] a = new int[n];
        for (int i = 0; i < n; i++) {
            a[i] = r.nextInt();
        }
        return a;
    }

    private static void assertSorts(ParallelSorter s, int[] input, int parallelism) {
        int[] expected = input.clone();
        Arrays.sort(expected);
        int[] actual = input.clone();

        s.sort(actual, parallelism);

        assertThat(actual).as("n=%d parallelism=%d", input.length, parallelism).isEqualTo(expected);
    }

    @Test
    void shouldHandleTinyInputs() {
        ParallelSorter s = sorter();

        for (int p : new int[] {1, 2, 8}) {
            assertSorts(s, new int[] {}, p);
            assertSorts(s, new int[] {42}, p);
            assertSorts(s, new int[] {2, 1}, p);
            assertSorts(s, new int[] {3, 1, 2}, p);
        }
    }

    @Test
    void shouldSortSpecialShapes() {
        ParallelSorter s = sorter();
        int n = 50_000;
        int[] sorted = new int[n];
        int[] reversed = new int[n];
        int[] equal = new int[n];
        int[] fewDistinct = new int[n];
        Random r = new Random(7);
        for (int i = 0; i < n; i++) {
            sorted[i] = i;
            reversed[i] = n - i;
            equal[i] = 5;
            fewDistinct[i] = r.nextInt(4);
        }

        for (int[] shape : new int[][] {sorted, reversed, equal, fewDistinct}) {
            assertSorts(s, shape, 4);
        }
    }

    @Test
    void shouldHandleExtremeValuesWithoutOverflow() {
        ParallelSorter s = sorter();
        int[] a = {Integer.MAX_VALUE, Integer.MIN_VALUE, 0, -1, 1, Integer.MAX_VALUE, Integer.MIN_VALUE};

        assertSorts(s, a, 1);
        assertSorts(s, a, 4);
    }

    @Test
    void shouldSortRandomArraysOfManySizesAndParallelisms() {
        ParallelSorter s = sorter();

        for (int n : new int[] {10, 1_000, 100_003, 1_000_000}) {
            for (int p : new int[] {1, 2, 4, 8}) {
                assertSorts(s, random(n, n * 31L + p), p);
            }
        }
    }

    @Test
    void shouldAcceptMoreThreadsThanElements() {
        ParallelSorter s = sorter();

        assertSorts(s, random(5, 1), 64);
        assertSorts(s, random(100, 2), 1_000);
    }

    @Test
    void shouldRejectNonPositiveParallelism() {
        ParallelSorter s = sorter();

        assertThatThrownBy(() -> s.sort(new int[] {2, 1}, 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> s.sort(new int[] {2, 1}, -3)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldBeUsableFromSeveralThreadsAtOnce() {
        ParallelSorter s = sorter();

        Concurrently.run(6, t -> assertSorts(s, random(200_000, 100 + t), 4));
    }

    @Test
    void shouldNotSpawnMoreThreadsThanAllowed() throws Exception {
        ParallelSorter s = sorter();
        int[] warmup = random(10_000, 1);
        s.sort(warmup, 2);

        assertThat(maxExtraThreadsWhileSorting(s, 1)).as("parallelism 1 needs no (or one) helper").isLessThanOrEqualTo(1);
        int extra = maxExtraThreadsWhileSorting(s, 4);
        assertThat(extra).as("a thread per split would explode here").isLessThanOrEqualTo(4 + 1);
    }

    @Test
    void shouldReallyRunInParallel() throws Exception {
        ParallelSorter s = sorter();
        s.sort(random(10_000, 1), 2);

        int extra = maxExtraThreadsWhileSorting(s, 4);

        assertThat(extra).as("no helper thread was ever seen: the sort looks sequential").isGreaterThanOrEqualTo(1);
    }

    @Test
    void shouldFinishALargeSortQuickly() {
        ParallelSorter s = sorter();
        int[] big = random(3_000_000, 99);

        Assertions.assertTimeoutPreemptively(Duration.ofSeconds(20), () -> s.sort(big, 4));

        for (int i = 1; i < big.length; i++) {
            if (big[i - 1] > big[i]) {
                throw new AssertionError("not sorted at index " + i);
            }
        }
    }

    @Test
    void shouldStayCorrectOver100Runs() {
        ParallelSorter s = sorter();
        Random r = new Random(2024);
        for (int run = 0; run < 100; run++) {
            int n = r.nextInt(20_000);
            int p = 1 + r.nextInt(8);
            assertSorts(s, random(n, run), p);
        }
    }

    /** Samples the number of live threads while a big sort runs; returns the peak minus the baseline. */
    private static int maxExtraThreadsWhileSorting(ParallelSorter s, int parallelism) throws Exception {
        int[] big = random(4_000_000, 5);
        AtomicBoolean stop = new AtomicBoolean();
        AtomicInteger peak = new AtomicInteger();
        Thread monitor = new Thread(() -> {
            while (!stop.get()) {
                peak.accumulateAndGet(Thread.getAllStackTraces().size(), Math::max);
                Thread.onSpinWait();
            }
        });
        monitor.setDaemon(true);
        monitor.start();
        Thread.sleep(50);
        int baseline = Thread.getAllStackTraces().size();
        peak.set(baseline);

        s.sort(big, parallelism);

        stop.set(true);
        monitor.join();
        return peak.get() - baseline;
    }
}

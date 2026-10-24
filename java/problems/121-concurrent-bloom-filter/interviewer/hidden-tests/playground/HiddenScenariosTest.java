package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.ConcurrentBloomFilter;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicLong;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    private static ConcurrentBloomFilter filter(int expectedItems, double rate) {
        return Candidate.of(() -> Solution.create(expectedItems, rate));
    }

    private static List<String> items(String prefix, int from, int to) {
        List<String> out = new ArrayList<>();
        for (int i = from; i < to; i++) {
            out.add(prefix + i);
        }
        return out;
    }

    @Test
    void shouldReportASaturatedFilterAsEverythingPossiblyPresent() {
        ConcurrentBloomFilter f = filter(1, 0.5); // m = 2, k = 1
        assertThat(f.bitCount()).isEqualTo(2);
        assertThat(f.hashCount()).isEqualTo(1);

        f.addAll(items("fill-", 0, 200));

        assertThat(f.setBits()).isEqualTo(2);
        assertThat(f.approximateSize()).isEqualTo(Long.MAX_VALUE);
        assertThat(f.mightContain("never-added")).isTrue();
        assertThat(f.add("one-more")).isFalse();
    }

    @Test
    void shouldStayConsistentWhenManyThreadsAddOverlappingBatches() {
        int threads = 16;
        List<String> pool = items("o-", 0, 6_000);
        ConcurrentBloomFilter concurrent = filter(6_000, 0.01);
        ConcurrentBloomFilter sequential = filter(6_000, 0.01);

        Concurrently.run(threads, t -> {
            List<String> mine = new ArrayList<>(pool);
            Collections.shuffle(mine, new java.util.Random(t));
            concurrent.addAll(mine);
        });
        sequential.addAll(pool);

        assertThat(concurrent.setBits()).isEqualTo(sequential.setBits());
        for (String s : pool) {
            assertThat(concurrent.mightContain(s)).isTrue();
        }
    }

    @Test
    void shouldKeepTheRateForSeveralParameterSets() {
        int[][] cases = {{2_000, 1}, {5_000, 5}, {1_000, 10}}; // expectedItems, rate in percent
        for (int[] c : cases) {
            double rate = c[1] / 100.0;
            ConcurrentBloomFilter f = filter(c[0], rate);
            f.addAll(items("in-", 0, c[0]));

            int fp = 0;
            for (int i = 0; i < 100_000; i++) {
                if (f.mightContain("out-" + i)) {
                    fp++;
                }
            }

            assertThat(fp).as("n=%d rate=%s", c[0], rate).isLessThanOrEqualTo((int) (3 * rate * 100_000));
        }
    }

    @Test
    void shouldEstimateTheSizeAcrossFillLevels() {
        ConcurrentBloomFilter f = filter(20_000, 0.01);
        int added = 0;
        for (int target : new int[] {100, 1_000, 5_000, 10_000, 20_000}) {
            f.addAll(items("g-", added, target));
            added = target;

            assertThat(f.approximateSize()).as("after %d", target).isBetween((long) (target * 0.95), (long) (target * 1.05));
        }
    }

    @Test
    void shouldHandleVeryLongAndOddItems() {
        ConcurrentBloomFilter f = filter(1_000, 1e-9);
        String longItem = "x".repeat(200_000);
        String longOther = "x".repeat(199_999) + "y";

        assertThat(f.add(longItem)).isTrue();
        assertThat(f.mightContain(longItem)).isTrue();
        assertThat(f.mightContain(longOther)).isFalse();
        assertThat(f.add("a\u0000b")).isTrue();
        assertThat(f.mightContain("a")).isFalse();
        assertThat(f.add("\n x \n")).isTrue();
        assertThat(f.mightContain("x")).isFalse();
    }

    @Test
    void shouldKeepAddedItemsVisibleWhileTheEstimateIsReadConcurrently() {
        ConcurrentBloomFilter f = filter(30_000, 0.01);
        AtomicLong lastSeen = new AtomicLong();
        AtomicBoolean decreased = new AtomicBoolean();

        Concurrently.run(5, t -> {
            if (t < 4) {
                for (int i = 0; i < 7_500; i++) {
                    f.add("e-" + t + "-" + i);
                }
            } else {
                long previous = 0;
                for (int i = 0; i < 2_000; i++) {
                    long now = f.approximateSize();
                    if (now < previous) {
                        decreased.set(true);
                    }
                    previous = now;
                    lastSeen.set(now);
                }
            }
        });

        assertThat(decreased.get()).as("estimate went down while only adding").isFalse();
        assertThat(f.approximateSize()).isBetween(28_000L, 32_000L);
    }

    @Test
    void shouldReportEachDistinctItemOnceAcrossThreadsInARoomyFilter() {
        int threads = 8;
        ConcurrentBloomFilter f = filter(4_000, 1e-9);
        AtomicInteger changed = new AtomicInteger();

        Concurrently.run(threads, t -> {
            for (int i = 0; i < 500; i++) {
                if (f.add("r-" + t + "-" + i)) {
                    changed.incrementAndGet();
                }
            }
        });

        assertThat(changed.get()).isEqualTo(threads * 500);
    }

    @Test
    void shouldWorkAgainAfterClearUnderConcurrentUse() {
        ConcurrentBloomFilter f = filter(10_000, 0.01);
        f.addAll(items("old-", 0, 5_000));
        f.clear();

        Concurrently.run(8, t -> {
            for (int i = 0; i < 1_000; i++) {
                f.add("new-" + t + "-" + i);
            }
        });

        for (int t = 0; t < 8; t++) {
            for (int i = 0; i < 1_000; i++) {
                assertThat(f.mightContain("new-" + t + "-" + i)).isTrue();
            }
        }
        assertThat(f.approximateSize()).isBetween(7_600L, 8_400L);
        int stillThere = 0;
        for (String s : items("old-", 0, 5_000)) {
            if (f.mightContain(s)) {
                stillThere++;
            }
        }
        assertThat(stillThere).isLessThan(300); // only false positives
    }
}

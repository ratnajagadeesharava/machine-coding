package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.VisitCounter;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.TreeMap;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    private static long used() {
        for (int i = 0; i < 3; i++) {
            System.gc();
        }
        Runtime r = Runtime.getRuntime();
        return r.totalMemory() - r.freeMemory();
    }

    private static long num(String line, int fromEnd) {
        String[] p = line.split(":");
        return Long.parseLong(p[p.length - fromEnd]);
    }

    @Test
    void shouldNotGrowWithTheNumberOfVisitsWhenVisitorsAreFew() {
        VisitCounter c = Candidate.of(Solution::create);
        String[] ids = new String[100];
        for (int i = 0; i < ids.length; i++) {
            ids[i] = "visitor-" + i;
        }
        c.visit("https://s.com/hot", ids[0]);
        long before = used();
        for (int i = 0; i < 30_000_000; i++) {
            c.visit("https://s.com/hot", ids[i % 100]);
        }
        long after = used();

        assertThat(c.totalVisits("https://s.com/hot")).isEqualTo(30_000_001L);
        assertThat(c.uniqueVisitors("https://s.com/hot")).isEqualTo(100);
        assertThat(after - before).as("heap growth in bytes").isLessThan(24L * 1024 * 1024);
    }

    @Test
    void shouldFoldManySpellingsOfOnePageIntoOneEntryFromAllThreads() {
        VisitCounter c = Candidate.of(Solution::create);
        String[] spellings = {
            "https://Shop.com/cart", "HTTPS://shop.COM/cart/", "https://shop.com/cart?x=1", "https://shop.com/cart#top",
            "  https://shop.com/cart/?a#b  ", "https://SHOP.com/cart//"
        };
        Concurrently.run(16, 60_000, t -> {
            for (int i = 0; i < 20_000; i++) {
                c.visit(spellings[(t + i) % spellings.length], "u" + (i % 64));
            }
        });

        assertThat(c.pageCount()).isEqualTo(1);
        assertThat(c.totalVisits("https://shop.com/cart")).isEqualTo(320_000);
        assertThat(c.uniqueVisitors("https://shop.com/cart")).isEqualTo(64);
        assertThat(c.snapshot()).containsExactly("https://shop.com/cart:320000:64");
    }

    @Test
    void shouldCountDistinctVisitorsExactlyWhenThereAreHundredsOfThousands() {
        VisitCounter c = Candidate.of(Solution::create);
        Concurrently.run(16, 60_000, t -> {
            // thread t visits ids t, t+16, ... (disjoint), and the shared ids 0..9999 as well
            for (int i = 0; i < 20_000; i++) {
                c.visit("https://s.com/big", "id-" + (t + 16 * i));
                c.visit("https://s.com/big", "shared-" + (i % 10_000));
            }
        });

        assertThat(c.uniqueVisitors("https://s.com/big")).isEqualTo(320_000 + 10_000);
        assertThat(c.totalVisits("https://s.com/big")).isEqualTo(640_000);
        assertThat(c.visitsBy("https://s.com/big", "shared-5")).isEqualTo(32);
        assertThat(c.visitsBy("https://s.com/big", "id-17")).isEqualTo(1);
    }

    @Test
    void shouldKeepSnapshotsMonotonicAndSerialWhileWritersRunTripleOrderedChains() {
        VisitCounter c = Candidate.of(Solution::create);
        // Each writer visits a, b, c in turn: any serial prefix has a >= b >= c >= a - 1.
        Concurrently.run(14, 60_000, t -> {
            if (t < 8) {
                for (int i = 0; i < 25_000; i++) {
                    c.visit("https://s.com/a" + t, "v" + (i % 7));
                    c.visit("https://s.com/b" + t, "v" + (i % 7));
                    c.visit("https://s.com/c" + t, "v" + (i % 7));
                }
                return;
            }
            Map<String, Long> last = new HashMap<>();
            for (int n = 0; n < 200; n++) {
                Map<String, Long> now = new TreeMap<>();
                for (String line : c.snapshot()) {
                    String url = line.substring(0, line.indexOf(':', 8));
                    now.put(url, num(line, 2));
                }
                now.forEach((u, v) -> assertThat(v).as(u).isGreaterThanOrEqualTo(last.getOrDefault(u, 0L)));
                for (int w = 0; w < 8; w++) {
                    long a = now.getOrDefault("https://s.com/a" + w, 0L);
                    long b = now.getOrDefault("https://s.com/b" + w, 0L);
                    long cc = now.getOrDefault("https://s.com/c" + w, 0L);
                    assertThat(a - b).as("a-b " + w).isBetween(0L, 1L);
                    assertThat(b - cc).as("b-c " + w).isBetween(0L, 1L);
                }
                last.putAll(now);
            }
        });

        assertThat(c.totalVisits()).isEqualTo(8L * 75_000);
    }

    @Test
    void shouldReturnWellFormedRankingsWhileCountsChange() {
        VisitCounter c = Candidate.of(Solution::create);
        Concurrently.run(16, 60_000, t -> {
            if (t < 12) {
                for (int i = 0; i < 20_000; i++) {
                    c.visit("https://s.com/p" + (i % 30), "v" + ((i + t) % 25));
                }
                return;
            }
            for (int n = 0; n < 300; n++) {
                List<String> top = c.topPages(5);
                assertThat(top).hasSizeLessThanOrEqualTo(5);
                long prev = Long.MAX_VALUE;
                String prevUrl = "";
                for (String line : top) {
                    long total = num(line, 1);
                    String url = line.substring(0, line.lastIndexOf(':'));
                    assertThat(total).isLessThanOrEqualTo(prev);
                    if (total == prev) {
                        assertThat(url.compareTo(prevUrl)).isPositive();
                    }
                    prev = total;
                    prevUrl = url;
                }
                List<String> who = c.topVisitors("https://s.com/p3", 4);
                assertThat(who).hasSizeLessThanOrEqualTo(4);
                long p = Long.MAX_VALUE;
                for (String line : who) {
                    assertThat(num(line, 1)).isLessThanOrEqualTo(p);
                    p = num(line, 1);
                }
            }
        });

        assertThat(c.totalVisits()).isEqualTo(240_000);
        assertThat(c.topPages(100)).hasSize(30);
        assertThat(c.topPages(1)).hasSize(1);
    }

    @Test
    void shouldKeepTotalsEqualToVisitorSumsAfterConcurrentResets() {
        VisitCounter c = Candidate.of(Solution::create);
        Concurrently.run(16, 60_000, t -> {
            for (int i = 0; i < 10_000; i++) {
                if (t % 4 == 3) {
                    if (i % 25 == 0) {
                        c.reset("https://s.com/p" + (i % 5));
                    }
                    if (i % 1000 == 0) {
                        c.resetAll();
                    }
                } else {
                    c.visit("https://s.com/p" + (i % 5), "v" + (i % 13));
                }
            }
        });

        Map<String, Long> sums = new TreeMap<>();
        for (String line : c.snapshotDetail()) {
            String noCount = line.substring(0, line.lastIndexOf(':'));
            sums.merge(noCount.substring(0, noCount.lastIndexOf(':')), num(line, 1), Long::sum);
        }
        List<String> expected = new ArrayList<>();
        sums.forEach((u, s) -> expected.add(u + ":" + s));
        List<String> totals = new ArrayList<>();
        for (String line : c.snapshot()) {
            totals.add(line.substring(0, line.lastIndexOf(':')));
        }
        assertThat(totals).isEqualTo(expected);
        assertThat(c.totalVisits()).isEqualTo(sums.values().stream().mapToLong(Long::longValue).sum());
    }

    @Test
    void shouldResetAndRevisitConcurrentlyWithoutNegativeOrTornCounts() {
        VisitCounter c = Candidate.of(Solution::create);
        Concurrently.run(8, 60_000, t -> {
            for (int i = 0; i < 20_000; i++) {
                if (t == 0) {
                    c.reset("https://s.com/hot");
                } else {
                    c.visit("https://s.com/hot", "v" + t);
                    long total = c.totalVisits("https://s.com/hot");
                    long unique = c.uniqueVisitors("https://s.com/hot");
                    assertThat(total).isGreaterThanOrEqualTo(0);
                    assertThat(unique).isLessThanOrEqualTo(7);
                }
            }
        });

        c.reset("https://s.com/hot");
        c.visit("https://s.com/hot", "last");
        assertThat(c.snapshot()).containsExactly("https://s.com/hot:1:1");
    }
}

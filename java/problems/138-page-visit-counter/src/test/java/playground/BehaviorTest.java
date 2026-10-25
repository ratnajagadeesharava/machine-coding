package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.VisitCounter;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Set;
import java.util.TreeMap;
import java.util.concurrent.atomic.AtomicBoolean;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private static final String A = "https://site.com/a";
    private static final String B = "https://site.com/b";

    private VisitCounter counter() {
        return Candidate.of(Solution::create);
    }

    private static long num(String line, int fromEnd) {
        String[] p = line.split(":");
        return Long.parseLong(p[p.length - fromEnd]);
    }

    @Test
    void shouldCountTotalVisitsPerPageAndReturnZeroForUnknownPages() {
        VisitCounter c = counter();
        c.visit(A, "u1");
        c.visit(A, "u2");
        c.visit(B, "u1");

        assertThat(c.totalVisits(A)).isEqualTo(2);
        assertThat(c.totalVisits(B)).isEqualTo(1);
        assertThat(c.totalVisits("https://site.com/never")).isZero();
        assertThat(c.uniqueVisitors("https://site.com/never")).isZero();
        assertThat(c.visitsBy("https://site.com/never", "u1")).isZero();
    }

    @Test
    void shouldCountUniqueVisitorsOnceNoMatterHowOftenTheyReturn() {
        VisitCounter c = counter();
        for (int i = 0; i < 5; i++) {
            c.visit(A, "u1");
        }
        c.visit(A, "u2");

        assertThat(c.totalVisits(A)).isEqualTo(6);
        assertThat(c.uniqueVisitors(A)).isEqualTo(2);
        assertThat(c.visitsBy(A, "u1")).isEqualTo(5);
        assertThat(c.visitsBy(A, "u2")).isEqualTo(1);
        assertThat(c.visitsBy(A, "u3")).isZero();
    }

    @Test
    void shouldKeepUniqueVisitorsPerPageIndependent() {
        VisitCounter c = counter();
        c.visit(A, "u1");
        c.visit(B, "u1");
        c.visit(B, "u2");

        assertThat(c.uniqueVisitors(A)).isEqualTo(1);
        assertThat(c.uniqueVisitors(B)).isEqualTo(2);
        assertThat(c.visitsBy(A, "u2")).isZero();
    }

    @Test
    void shouldTreatVisitorIdsAsCaseSensitiveAndUntrimmed() {
        VisitCounter c = counter();
        c.visit(A, "Bob");
        c.visit(A, "bob");
        c.visit(A, " bob");

        assertThat(c.uniqueVisitors(A)).isEqualTo(3);
        assertThat(c.visitsBy(A, "bob")).isEqualTo(1);
        assertThat(c.visitsBy(A, " bob")).isEqualTo(1);
    }

    @Test
    void shouldNormaliseSchemeHostQueryFragmentAndWhitespaceToOnePage() {
        VisitCounter c = counter();
        c.visit("https://example.com/about", "u1");
        c.visit("  HTTPS://Example.COM/about  ", "u2");
        c.visit("https://example.com/about?utm=1&x=2", "u3");
        c.visit("https://example.com/about#team", "u4");
        c.visit("https://example.com/about/?a=b#c", "u5");

        assertThat(c.pageCount()).isEqualTo(1);
        assertThat(c.totalVisits("https://EXAMPLE.com/about")).isEqualTo(5);
        assertThat(c.uniqueVisitors("https://example.com/about/")).isEqualTo(5);
        assertThat(c.topPages(5)).containsExactly("https://example.com/about:5");
    }

    @Test
    void shouldKeepPathCaseAndTreatSchemePortAndPathAsDifferentPages() {
        VisitCounter c = counter();
        c.visit("https://example.com/About", "u");
        c.visit("https://example.com/about", "u");
        c.visit("http://example.com/about", "u");
        c.visit("https://example.com:8443/about", "u");
        c.visit("https://example.com/about/team", "u");

        assertThat(c.pageCount()).isEqualTo(5);
        assertThat(c.totalVisits("https://example.com/About")).isEqualTo(1);
    }

    @Test
    void shouldNormaliseTheRootPathToASingleSlash() {
        VisitCounter c = counter();
        c.visit("https://Example.com", "u1");
        c.visit("https://example.com/", "u2");
        c.visit("https://example.com//", "u3");
        c.visit("https://example.com/?q=1", "u4");

        assertThat(c.pageCount()).isEqualTo(1);
        assertThat(c.topPages(1)).containsExactly("https://example.com/:4");
    }

    @Test
    void shouldStripEveryTrailingSlashOfANonRootPath() {
        VisitCounter c = counter();
        c.visit("https://example.com/docs///", "u1");
        c.visit("https://example.com/docs", "u2");

        assertThat(c.topPages(3)).containsExactly("https://example.com/docs:2");
    }

    @Test
    void shouldAcceptUrlsWithoutAScheme() {
        VisitCounter c = counter();
        c.visit("Example.com/Pricing/", "u1");
        c.visit("example.com/Pricing?x=1", "u2");

        assertThat(c.topPages(3)).containsExactly("example.com/Pricing:2");
    }

    @Test
    void shouldRejectBlankNullAndHostlessUrlsAndBlankVisitors() {
        VisitCounter c = counter();
        c.visit(A, "u1");

        assertThatThrownBy(() -> c.visit(null, "u")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.visit("   ", "u")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.visit("", "u")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.visit("?a=1", "u")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.visit("https:///path", "u")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.visit(A, null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.visit(A, " ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.totalVisits(" ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.uniqueVisitors(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.visitsBy(" ", "u")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.visitsBy(A, " ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.topVisitors(" ", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.reset(" ")).isInstanceOf(IllegalArgumentException.class);

        assertThat(c.totalVisits()).isEqualTo(1);
        assertThat(c.pageCount()).isEqualTo(1);
    }

    @Test
    void shouldRankTopPagesByTotalThenUrl() {
        VisitCounter c = counter();
        for (int i = 0; i < 3; i++) c.visit("https://s.com/z", "u" + i);
        for (int i = 0; i < 3; i++) c.visit("https://s.com/m", "u" + i);
        for (int i = 0; i < 5; i++) c.visit("https://s.com/q", "u" + i);
        c.visit("https://s.com/a", "u0");

        assertThat(c.topPages(10))
                .containsExactly("https://s.com/q:5", "https://s.com/m:3", "https://s.com/z:3", "https://s.com/a:1");
        assertThat(c.topPages(2)).containsExactly("https://s.com/q:5", "https://s.com/m:3");
        assertThat(c.topPages(1)).containsExactly("https://s.com/q:5");
    }

    @Test
    void shouldRejectNonPositiveKAndReturnEmptyListsWhenNothingIsRecorded() {
        VisitCounter c = counter();

        assertThat(c.topPages(3)).isEmpty();
        assertThat(c.topVisitors(A, 3)).isEmpty();
        assertThatThrownBy(() -> c.topPages(0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.topPages(-1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.topVisitors(A, 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> c.topVisitors(A, -5)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRankTopVisitorsByCountThenId() {
        VisitCounter c = counter();
        for (int i = 0; i < 4; i++) c.visit(A, "zed");
        for (int i = 0; i < 4; i++) c.visit(A, "amy");
        for (int i = 0; i < 7; i++) c.visit(A, "kim");
        c.visit(A, "bob");
        c.visit(B, "other");

        assertThat(c.topVisitors(A, 10)).containsExactly("kim:7", "amy:4", "zed:4", "bob:1");
        assertThat(c.topVisitors(A + "/?x=1", 2)).containsExactly("kim:7", "amy:4");
        assertThat(c.topVisitors(B, 5)).containsExactly("other:1");
    }

    @Test
    void shouldReportGrandTotalAndPageCountAcrossPages() {
        VisitCounter c = counter();
        assertThat(c.totalVisits()).isZero();
        assertThat(c.pageCount()).isZero();

        c.visit(A, "u1");
        c.visit(A, "u1");
        c.visit(B, "u2");

        assertThat(c.totalVisits()).isEqualTo(3);
        assertThat(c.pageCount()).isEqualTo(2);
    }

    @Test
    void shouldResetOnePageAndReportWhetherItExisted() {
        VisitCounter c = counter();
        c.visit(A, "u1");
        c.visit(A, "u2");
        c.visit(B, "u1");

        assertThat(c.reset(A + "/?utm=1")).isTrue();
        assertThat(c.reset(A)).isFalse();
        assertThat(c.reset("https://site.com/never")).isFalse();

        assertThat(c.totalVisits(A)).isZero();
        assertThat(c.uniqueVisitors(A)).isZero();
        assertThat(c.visitsBy(A, "u1")).isZero();
        assertThat(c.totalVisits(B)).isEqualTo(1);
        assertThat(c.pageCount()).isEqualTo(1);
        assertThat(c.totalVisits()).isEqualTo(1);
        assertThat(c.topPages(5)).containsExactly(B + ":1");
    }

    @Test
    void shouldStartAPageFromScratchAfterItWasReset() {
        VisitCounter c = counter();
        c.visit(A, "u1");
        c.visit(A, "u1");
        c.reset(A);
        c.visit(A, "u1");

        assertThat(c.totalVisits(A)).isEqualTo(1);
        assertThat(c.uniqueVisitors(A)).isEqualTo(1);
        assertThat(c.visitsBy(A, "u1")).isEqualTo(1);
    }

    @Test
    void shouldResetEverything() {
        VisitCounter c = counter();
        c.visit(A, "u1");
        c.visit(B, "u2");
        c.resetAll();

        assertThat(c.pageCount()).isZero();
        assertThat(c.totalVisits()).isZero();
        assertThat(c.topPages(3)).isEmpty();
        assertThat(c.snapshot()).isEmpty();
        assertThat(c.reset(A)).isFalse();

        c.visit(A, "u9");
        assertThat(c.totalVisits()).isEqualTo(1);
    }

    @Test
    void shouldExportASnapshotSortedByUrlWithTotalAndUnique() {
        VisitCounter c = counter();
        c.visit("https://s.com/b", "u1");
        c.visit("https://s.com/b", "u1");
        c.visit("https://s.com/a", "u1");
        c.visit("https://s.com/a", "u2");
        c.visit("https://s.com/a", "u3");
        c.visit("https://s.com/a/x", "u3");

        assertThat(c.snapshot())
                .containsExactly("https://s.com/a:3:3", "https://s.com/a/x:1:1", "https://s.com/b:2:1");
        assertThat(c.snapshotDetail())
                .containsExactly(
                        "https://s.com/a:u1:1",
                        "https://s.com/a:u2:1",
                        "https://s.com/a:u3:1",
                        "https://s.com/a/x:u3:1",
                        "https://s.com/b:u1:2");
    }

    @Test
    void shouldNotChangeASnapshotThatWasAlreadyTaken() {
        VisitCounter c = counter();
        c.visit(A, "u1");
        List<String> before = c.snapshot();
        List<String> beforeDetail = c.snapshotDetail();
        c.visit(A, "u2");
        c.visit(B, "u3");
        c.reset(A);

        assertThat(before).containsExactly(A + ":1:1");
        assertThat(beforeDetail).containsExactly(A + ":u1:1");
        assertThat(c.snapshot()).containsExactly(B + ":1:1");
    }

    @Test
    void shouldCountEveryVisitExactlyUnderSixteenWritingThreads() {
        VisitCounter c = counter();
        int threads = 16;
        int perThread = 20_000;
        Concurrently.run(threads, 60_000, t -> {
            for (int i = 0; i < perThread; i++) {
                c.visit(spelling(url(t, i), i), visitor(t, i));
            }
        });

        Map<String, Long> total = new HashMap<>();
        Map<String, Set<String>> unique = new HashMap<>();
        Map<String, Long> by = new HashMap<>();
        for (int t = 0; t < threads; t++) {
            for (int i = 0; i < perThread; i++) {
                total.merge(url(t, i), 1L, Long::sum);
                unique.computeIfAbsent(url(t, i), k -> new HashSet<>()).add(visitor(t, i));
                by.merge(url(t, i) + "|" + visitor(t, i), 1L, Long::sum);
            }
        }
        assertThat(c.pageCount()).isEqualTo(total.size());
        assertThat(c.totalVisits()).isEqualTo((long) threads * perThread);
        for (String u : total.keySet()) {
            assertThat(c.totalVisits(u)).as(u).isEqualTo(total.get(u));
            assertThat(c.uniqueVisitors(u)).as(u).isEqualTo(unique.get(u).size());
        }
        by.forEach((k, v) -> {
            String[] p = k.split("\\|");
            assertThat(c.visitsBy(p[0], p[1])).as(k).isEqualTo(v);
        });
    }

    private static String url(int t, int i) {
        return "https://site.com/p" + ((t * 31 + i * 7) % 50);
    }

    private static String visitor(int t, int i) {
        return "v" + ((t * 131 + i * 17) % 1000);
    }

    /** Same page, varying spelling: exercises normalisation from many threads. */
    private static String spelling(String url, int i) {
        return switch (i % 3) {
            case 0 -> url;
            case 1 -> url.replace("https://site.com", "HTTPS://SITE.com") + "/?n=" + i;
            default -> "  " + url + "#f" + i + " ";
        };
    }

    @Test
    void shouldTakeConsistentSnapshotsWhileSixteenThreadsKeepVisiting() {
        VisitCounter c = counter();
        int writers = 12;
        AtomicBoolean stop = new AtomicBoolean();
        // Writer w alternates "x<w>" then "y<w>": in any serial prefix 0 <= x - y <= 1.
        List<List<String>> seen = Concurrently.collect(writers + 4, 60_000, t -> {
            List<String> snaps = new ArrayList<>();
            if (t < writers) {
                for (int i = 0; i < 20_000; i++) {
                    c.visit("https://s.com/x" + t, "v" + (i % 50));
                    c.visit("https://s.com/y" + t, "v" + (i % 50));
                }
                return snaps;
            }
            long last = 0;
            for (int n = 0; n < 150; n++) {
                List<String> snap = c.snapshot();
                long sum = 0;
                for (String line : snap) {
                    assertThat(num(line, 2)).as(line).isGreaterThanOrEqualTo(num(line, 1));
                    sum += num(line, 2);
                }
                assertThat(sum).isGreaterThanOrEqualTo(last);
                last = sum;
                snaps.addAll(snap);
                Thread.yield();
            }
            return snaps;
        });

        for (int s = writers; s < writers + 4; s++) {
            List<String> all = seen.get(s);
            // snapshots were concatenated; check pair invariant in each by grouping consecutive lines per snapshot
            Map<String, Long> cur = new HashMap<>();
            for (String line : all) {
                String name = line.substring(0, line.indexOf(':', 8));
                if (cur.containsKey(name)) {
                    checkPairs(cur);
                    cur.clear();
                }
                cur.put(name, num(line, 2));
            }
            checkPairs(cur);
        }
        assertThat(c.totalVisits()).isEqualTo((long) writers * 40_000);
    }

    private static void checkPairs(Map<String, Long> snap) {
        for (int w = 0; w < 64; w++) {
            Long x = snap.get("https://s.com/x" + w);
            Long y = snap.get("https://s.com/y" + w);
            long xv = x == null ? 0 : x;
            long yv = y == null ? 0 : y;
            assertThat(xv - yv).as("pair " + w + " x=" + xv + " y=" + yv).isBetween(0L, 1L);
        }
    }

    @Test
    void shouldGiveDetailSnapshotsWhosePerVisitorCountsAddUpWhileVisitsContinue() {
        VisitCounter c = counter();
        Concurrently.run(10, 60_000, t -> {
            if (t < 8) {
                for (int i = 0; i < 20_000; i++) {
                    c.visit("https://s.com/x" + t, "v" + (i % 40));
                    c.visit("https://s.com/y" + t, "v" + (i % 40));
                }
                return;
            }
            for (int n = 0; n < 100; n++) {
                Map<String, Long> sums = new TreeMap<>();
                for (String line : c.snapshotDetail()) {
                    String noCount = line.substring(0, line.lastIndexOf(':'));
                    String url = noCount.substring(0, noCount.lastIndexOf(':'));
                    sums.merge(url, num(line, 1), Long::sum);
                }
                for (int w = 0; w < 8; w++) {
                    long x = sums.getOrDefault("https://s.com/x" + w, 0L);
                    long y = sums.getOrDefault("https://s.com/y" + w, 0L);
                    assertThat(x - y).as("pair " + w).isBetween(0L, 1L);
                }
            }
        });

        for (int w = 0; w < 8; w++) {
            assertThat(c.totalVisits("https://s.com/x" + w)).isEqualTo(20_000);
            assertThat(c.uniqueVisitors("https://s.com/y" + w)).isEqualTo(40);
        }
    }

    @Test
    void shouldHandleOneMillionVisitsFromAHundredVisitorsOnOnePage() {
        VisitCounter c = counter();
        for (int i = 0; i < 1_000_000; i++) {
            c.visit(A, "visitor-" + (i % 100));
        }

        assertThat(c.totalVisits(A)).isEqualTo(1_000_000);
        assertThat(c.uniqueVisitors(A)).isEqualTo(100);
        assertThat(c.visitsBy(A, "visitor-7")).isEqualTo(10_000);
        assertThat(c.topVisitors(A, 2)).containsExactly("visitor-0:10000", "visitor-1:10000");
    }

    @Test
    void shouldNotDeadlockOrLoseCountsWhenReadersResettersAndWritersMix() {
        VisitCounter c = counter();
        Concurrently.run(16, 60_000, t -> {
            for (int i = 0; i < 5_000; i++) {
                switch (t % 4) {
                    case 0 -> c.visit("https://s.com/p" + (i % 10), "v" + (i % 20));
                    case 1 -> {
                        c.topPages(3);
                        c.totalVisits();
                        c.snapshot();
                    }
                    case 2 -> {
                        c.topVisitors("https://s.com/p" + (i % 10), 3);
                        c.uniqueVisitors("https://s.com/p" + (i % 10));
                        c.snapshotDetail();
                    }
                    default -> {
                        if (i % 50 == 0) c.reset("https://s.com/p" + (i % 10));
                    }
                }
            }
        });

        long sum = 0;
        for (String line : c.snapshot()) {
            sum += num(line, 2);
        }
        assertThat(c.totalVisits()).isEqualTo(sum);
        c.visit(A, "after");
        assertThat(c.totalVisits(A)).isEqualTo(1);
    }
}

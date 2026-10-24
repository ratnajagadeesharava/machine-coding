package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.BrowserHistory;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class BehaviorTest {

    private static final String BLANK = "about:blank";

    private final FakeClock clock = new FakeClock(1_000L);

    private BrowserHistory browser(int capacity) {
        return Candidate.of(() -> Solution.create(capacity, clock));
    }

    private BrowserHistory browser() {
        return browser(1_000);
    }

    /** Visits the urls one after another, advancing the clock by 10 ms after each visit. */
    private void visitAll(BrowserHistory b, int tab, String... urls) {
        for (String u : urls) {
            b.visit(tab, u);
            clock.advance(10);
        }
    }

    @Test
    void shouldNumberTabsFromOneAndOpenThemOnABlankPage() {
        BrowserHistory b = browser();

        assertThat(b.openTab()).isEqualTo(1);
        assertThat(b.openTab()).isEqualTo(2);
        assertThat(b.currentUrl(1)).isEqualTo(BLANK);
        assertThat(b.canGoBack(1)).isFalse();
        assertThat(b.canGoForward(1)).isFalse();
        assertThat(b.history(10)).isEmpty();
    }

    @Test
    void shouldNotReuseIdsOfClosedTabs() {
        BrowserHistory b = browser();
        int t1 = b.openTab();
        b.closeTab(t1);

        assertThat(b.openTab()).isEqualTo(2);
        assertThatThrownBy(() -> b.currentUrl(t1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.closeTab(t1)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldNavigateAndShowTheLatestPage() {
        BrowserHistory b = browser();
        int t = b.openTab();
        visitAll(b, t, "https://a.com", "https://b.com");

        assertThat(b.currentUrl(t)).isEqualTo("https://b.com");
        assertThat(b.canGoBack(t)).isTrue();
        assertThat(b.canGoForward(t)).isFalse();
    }

    @Test
    void shouldGoBackAndForwardByTheRequestedSteps() {
        BrowserHistory b = browser();
        int t = b.openTab();
        visitAll(b, t, "a", "b", "c", "d");

        assertThat(b.back(t, 1)).isEqualTo("c");
        assertThat(b.back(t, 2)).isEqualTo("a");
        assertThat(b.forward(t, 1)).isEqualTo("b");
        assertThat(b.forward(t, 2)).isEqualTo("d");
        assertThat(b.canGoForward(t)).isFalse();
        assertThat(b.back(t, 0)).isEqualTo("d");
        assertThat(b.forward(t, 0)).isEqualTo("d");
    }

    @Test
    void shouldStopAtTheEndsInsteadOfFailing() {
        BrowserHistory b = browser();
        int t = b.openTab();
        visitAll(b, t, "a", "b");

        assertThat(b.back(t, 99)).isEqualTo(BLANK);
        assertThat(b.canGoBack(t)).isFalse();
        assertThat(b.back(t, 1)).isEqualTo(BLANK);
        assertThat(b.forward(t, Integer.MAX_VALUE)).isEqualTo("b");
        assertThat(b.forward(t, 1)).isEqualTo("b");
        assertThat(b.back(t, Integer.MAX_VALUE)).isEqualTo(BLANK);
    }

    @Test
    void shouldDiscardForwardEntriesWhenVisitingFromTheMiddle() {
        BrowserHistory b = browser();
        int t = b.openTab();
        visitAll(b, t, "a", "b", "c");
        b.back(t, 2);

        b.visit(t, "x");

        assertThat(b.currentUrl(t)).isEqualTo("x");
        assertThat(b.canGoForward(t)).isFalse();
        assertThat(b.forward(t, 5)).isEqualTo("x");
        assertThat(b.back(t, 1)).isEqualTo("a");
        assertThat(b.back(t, 1)).isEqualTo(BLANK);
    }

    @Test
    void shouldNotAddASessionEntryForTheCurrentUrlButStillRecordTheVisit() {
        BrowserHistory b = browser();
        int t = b.openTab();
        visitAll(b, t, "a", "a", "a");

        assertThat(b.back(t, 1)).isEqualTo(BLANK);
        b.forward(t, 1);
        assertThat(b.currentUrl(t)).isEqualTo("a");
        assertThat(b.history(10)).hasSize(3);
    }

    @Test
    void shouldKeepForwardEntriesWhenRevisitingTheCurrentUrl() {
        BrowserHistory b = browser();
        int t = b.openTab();
        visitAll(b, t, "a", "b");
        b.back(t, 1);

        b.visit(t, "a");

        assertThat(b.canGoForward(t)).isTrue();
        assertThat(b.forward(t, 1)).isEqualTo("b");
    }

    @Test
    void shouldTreatTheBlankPageAsAnOrdinaryUrlWhenVisited() {
        BrowserHistory b = browser();
        int t = b.openTab();

        b.visit(t, BLANK);

        assertThat(b.canGoBack(t)).isFalse();
        assertThat(b.history(5)).containsExactly("1000|about:blank|1");
    }

    @Test
    void shouldKeepTabsIndependent() {
        BrowserHistory b = browser();
        int t1 = b.openTab();
        int t2 = b.openTab();
        visitAll(b, t1, "a", "b");
        visitAll(b, t2, "x");

        b.back(t1, 1);

        assertThat(b.currentUrl(t1)).isEqualTo("a");
        assertThat(b.currentUrl(t2)).isEqualTo("x");
        assertThat(b.canGoBack(t2)).isTrue();
        assertThat(b.canGoForward(t2)).isFalse();
    }

    @Test
    void shouldRecordGlobalVisitsMostRecentFirstWithTimeUrlAndTab() {
        BrowserHistory b = browser();
        int t1 = b.openTab();
        int t2 = b.openTab();
        b.visit(t1, "https://a.com");
        clock.advance(5);
        b.visit(t2, "https://b.com");
        clock.advance(5);
        b.visit(t1, "https://c.com");

        assertThat(b.history(10))
                .containsExactly("1010|https://c.com|1", "1005|https://b.com|2", "1000|https://a.com|1");
        assertThat(b.history(2)).containsExactly("1010|https://c.com|1", "1005|https://b.com|2");
        assertThat(b.history(0)).isEmpty();
    }

    @Test
    void shouldNotRecordNavigationInTheGlobalHistory() {
        BrowserHistory b = browser();
        int t = b.openTab();
        visitAll(b, t, "a", "b", "c");

        b.back(t, 2);
        b.forward(t, 1);

        assertThat(b.history(10)).hasSize(3);
    }

    @Test
    void shouldReturnEntriesInAnInclusiveTimeRange() {
        BrowserHistory b = browser();
        int t = b.openTab();
        visitAll(b, t, "a", "b", "c", "d"); // 1000, 1010, 1020, 1030

        assertThat(b.historyBetween(1010, 1020)).containsExactly("1020|c|1", "1010|b|1");
        assertThat(b.historyBetween(1011, 1019)).isEmpty();
        assertThat(b.historyBetween(1030, 1030)).containsExactly("1030|d|1");
        assertThat(b.historyBetween(0, 5_000)).hasSize(4);
    }

    @Test
    void shouldSearchCaseInsensitivelyListingEachUrlOnceByLatestVisit() {
        BrowserHistory b = browser();
        int t = b.openTab();
        visitAll(b, t, "https://Docs.example.com/a", "https://news.com", "https://example.org", "https://docs.example.com/b");
        visitAll(b, t, "https://Docs.example.com/a");

        assertThat(b.search("EXAMPLE"))
                .containsExactly("https://Docs.example.com/a", "https://docs.example.com/b", "https://example.org");
        assertThat(b.search("docs.example.com/a")).containsExactly("https://Docs.example.com/a");
        assertThat(b.search("zzz")).isEmpty();
    }

    @Test
    void shouldRankMostVisitedByCountThenLatestVisitThenUrl() {
        BrowserHistory b = browser();
        int t = b.openTab();
        visitAll(b, t, "p", "q", "r");   // times 1000, 1010, 1020
        visitAll(b, t, "q", "p");        // 1030, 1040
        visitAll(b, t, "z");             // 1050
        visitAll(b, t, "y");             // 1060

        // p: 2 (latest 1040), q: 2 (latest 1030), then r, z, y with 1 each: latest 1020, 1050, 1060
        assertThat(b.mostVisited(10)).containsExactly("p", "q", "y", "z", "r");
        assertThat(b.mostVisited(2)).containsExactly("p", "q");
        assertThat(b.mostVisited(0)).isEmpty();
    }

    @Test
    void shouldBreakMostVisitedTiesByUrlWhenLatestVisitTimesAreEqual() {
        BrowserHistory b = browser();
        int t1 = b.openTab();
        int t2 = b.openTab();
        b.visit(t1, "b.com");
        b.visit(t2, "a.com");
        b.visit(t1, "c.com");

        assertThat(b.mostVisited(3)).containsExactly("a.com", "b.com", "c.com");
    }

    @Test
    void shouldDeleteAUrlFromTheGlobalHistoryButNotFromOpenTabs() {
        BrowserHistory b = browser();
        int t1 = b.openTab();
        int t2 = b.openTab();
        visitAll(b, t1, "a", "b");
        visitAll(b, t2, "a");

        assertThat(b.deleteFromHistory("a")).isEqualTo(2);
        assertThat(b.deleteFromHistory("a")).isZero();
        assertThat(b.deleteFromHistory("never")).isZero();

        assertThat(b.history(10)).containsExactly("1010|b|1");
        assertThat(b.search("a")).isEmpty();
        assertThat(b.back(t1, 1)).isEqualTo("a");
        assertThat(b.currentUrl(t2)).isEqualTo("a");
    }

    @Test
    void shouldClearEntriesFromAGivenTimeOnwards() {
        BrowserHistory b = browser();
        int t = b.openTab();
        visitAll(b, t, "a", "b", "c", "d"); // 1000, 1010, 1020, 1030

        assertThat(b.clearHistory(1020)).isEqualTo(2);
        assertThat(b.history(10)).containsExactly("1010|b|1", "1000|a|1");
        assertThat(b.clearHistory(5_000)).isZero();
        assertThat(b.currentUrl(t)).isEqualTo("d");
        assertThat(b.clearHistory(0)).isEqualTo(2);
        assertThat(b.history(10)).isEmpty();
    }

    @Test
    void shouldDropTheOldestEntriesBeyondTheCapacity() {
        BrowserHistory b = browser(3);
        int t = b.openTab();
        visitAll(b, t, "a", "b", "c", "d", "e");

        assertThat(b.history(10)).containsExactly("1040|e|1", "1030|d|1", "1020|c|1");
        assertThat(b.search("a")).isEmpty();
        assertThat(b.mostVisited(10)).containsExactly("e", "d", "c");
        assertThat(b.back(t, 4)).isEqualTo("a"); // tabs are not limited by the capacity
    }

    @Test
    void shouldFreeCapacityWhenEntriesAreDeleted() {
        BrowserHistory b = browser(4);
        int t = b.openTab();
        visitAll(b, t, "a", "a", "b", "a"); // 4 entries
        assertThat(b.mostVisited(1)).containsExactly("a");

        b.deleteFromHistory("a");
        visitAll(b, t, "c", "d", "e", "f"); // room for four again, "b" is pushed out

        assertThat(b.history(10)).hasSize(4);
        assertThat(b.search("b")).isEmpty();
    }

    @Test
    void shouldKeepHistoryEntriesOfClosedTabs() {
        BrowserHistory b = browser();
        int t = b.openTab();
        visitAll(b, t, "a");

        b.closeTab(t);

        assertThat(b.history(10)).containsExactly("1000|a|1");
    }

    @Test
    void shouldRejectInvalidInput() {
        BrowserHistory b = browser();
        int t = b.openTab();
        b.visit(t, "a");

        assertThatThrownBy(() -> b.visit(t, null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.visit(t, "")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.visit(t, "  \t")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.visit(99, "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.visit(0, "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.back(t, -1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.forward(t, -1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.back(99, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.forward(99, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.canGoBack(99)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.canGoForward(99)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.closeTab(99)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.history(-1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.historyBetween(10, 9)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.search(" ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.search(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.mostVisited(-1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> b.deleteFromHistory(" ")).isInstanceOf(IllegalArgumentException.class);

        assertThat(b.history(10)).hasSize(1);
        assertThat(b.currentUrl(t)).isEqualTo("a");
    }

    @Test
    void shouldRejectInvalidConfiguration() {
        browser(); // proves the factory is wired

        assertThatThrownBy(() -> Solution.create(0, clock)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(-3, clock)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(10, null)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldHandOutUniqueTabIdsToConcurrentCallers() {
        BrowserHistory b = browser();

        List<Integer> ids = Concurrently.collect(16, t -> {
            List<Integer> mine = new ArrayList<>();
            for (int i = 0; i < 100; i++) {
                mine.add(b.openTab());
            }
            return mine;
        }).stream().flatMap(List::stream).toList();

        assertThat(new HashSet<>(ids)).hasSize(1_600);
        assertThat(ids).allMatch(id -> id >= 1 && id <= 1_600);
    }

    @Test
    void shouldLoseNoVisitsWhenManyThreadsBrowseAtOnce() {
        BrowserHistory b = browser(100_000);
        int[] tabs = new int[8];
        for (int i = 0; i < tabs.length; i++) {
            tabs[i] = b.openTab();
        }

        Concurrently.run(8, 30_000, t -> {
            for (int i = 0; i < 500; i++) {
                b.visit(tabs[t], "https://site" + t + ".com/" + i);
                if (i % 5 == 4) {
                    b.back(tabs[t], 2);
                }
            }
        });

        assertThat(b.history(100_000)).hasSize(4_000);
        Set<String> seen = new HashSet<>(b.history(100_000));
        assertThat(seen).hasSize(4_000);
        for (int t = 0; t < 8; t++) {
            assertThat(b.search("site" + t + ".com/")).hasSize(500);
        }
    }

    @Test
    void shouldKeepTheCapacityUnderConcurrentVisitsAndDeletes() {
        BrowserHistory b = browser(50);
        int t0 = b.openTab();
        int t1 = b.openTab();

        Concurrently.run(8, 30_000, t -> {
            for (int i = 0; i < 500; i++) {
                b.visit(t % 2 == 0 ? t0 : t1, "u" + (i % 20));
                if (t == 7 && i % 50 == 0) {
                    b.deleteFromHistory("u3");
                }
                b.history(10);
            }
        });

        assertThat(b.history(1_000)).hasSizeLessThanOrEqualTo(50);
        assertThat(b.history(1_000)).hasSizeGreaterThan(40);
    }
}

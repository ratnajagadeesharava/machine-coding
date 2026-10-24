package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.BrowserHistory;
import java.util.List;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class HiddenScenariosTest {

    private final FakeClock clock = new FakeClock(5_000L);

    private BrowserHistory browser(int capacity) {
        return Candidate.of(() -> Solution.create(capacity, clock));
    }

    @Test
    void shouldTruncateForwardEntriesEvenWhenTheNewUrlEqualsAForwardEntry() {
        BrowserHistory b = browser(100);
        int t = b.openTab();
        b.visit(t, "a");
        b.visit(t, "b");
        b.visit(t, "c");
        b.back(t, 2);

        b.visit(t, "b"); // not the current url ("a"): a brand new entry, "b" and "c" ahead of it are gone

        assertThat(b.currentUrl(t)).isEqualTo("b");
        assertThat(b.canGoForward(t)).isFalse();
        assertThat(b.back(t, 1)).isEqualTo("a");
        assertThat(b.back(t, 1)).isEqualTo("about:blank");
        assertThat(b.forward(t, 10)).isEqualTo("b");
    }

    @Test
    void shouldOnlySuppressDuplicatesAgainstTheCurrentEntryNotAgainstEarlierOnes() {
        BrowserHistory b = browser(100);
        int t = b.openTab();
        b.visit(t, "a");
        b.visit(t, "b");
        b.visit(t, "a"); // "a" is not the current page: a third entry

        assertThat(b.back(t, 1)).isEqualTo("b");
        assertThat(b.back(t, 1)).isEqualTo("a");
        assertThat(b.back(t, 1)).isEqualTo("about:blank");
        assertThat(b.canGoBack(t)).isFalse();
    }

    @Test
    void shouldKeepInsertionOrderForEntriesWithTheSameTimestamp() {
        BrowserHistory b = browser(100);
        int t1 = b.openTab();
        int t2 = b.openTab();
        b.visit(t1, "x");
        b.visit(t2, "y");
        b.visit(t1, "z");

        assertThat(b.history(10)).containsExactly("5000|z|1", "5000|y|2", "5000|x|1");
        assertThat(b.historyBetween(5000, 5000)).containsExactly("5000|z|1", "5000|y|2", "5000|x|1");
    }

    @Test
    void shouldMakeCountsReflectOnlyRetainedEntries() {
        BrowserHistory b = browser(5);
        int t = b.openTab();
        for (String u : new String[] {"a", "a", "a", "b", "c"}) {
            b.visit(t, u);
            clock.advance(1);
        }
        assertThat(b.mostVisited(3)).containsExactly("a", "c", "b");

        b.visit(t, "b"); // pushes out the first "a": a=2, b=2, c=1
        clock.advance(1);
        assertThat(b.mostVisited(3)).containsExactly("b", "a", "c");

        b.visit(t, "d"); // pushes out the second "a": a=1
        assertThat(b.mostVisited(10)).containsExactly("b", "d", "c", "a");
        assertThat(b.deleteFromHistory("b")).isEqualTo(2);
        assertThat(b.mostVisited(10)).containsExactly("d", "c", "a");
    }

    @Test
    void shouldAcceptNewEntriesAfterClearingEverything() {
        BrowserHistory b = browser(3);
        int t = b.openTab();
        for (String u : new String[] {"a", "b", "c", "d"}) {
            b.visit(t, u);
            clock.advance(1);
        }
        assertThat(b.clearHistory(0)).isEqualTo(3);
        assertThat(b.history(10)).isEmpty();
        assertThat(b.mostVisited(5)).isEmpty();

        b.visit(t, "e");
        b.visit(t, "f");

        assertThat(b.history(10)).containsExactly("5004|f|1", "5004|e|1");
        assertThat(b.back(t, 100)).isEqualTo("about:blank");
    }

    @Test
    void shouldBehaveWithCapacityOne() {
        BrowserHistory b = browser(1);
        int t = b.openTab();
        b.visit(t, "a");
        b.visit(t, "b");

        assertThat(b.history(10)).containsExactly("5000|b|1");
        assertThat(b.mostVisited(5)).containsExactly("b");
        assertThat(b.back(t, 1)).isEqualTo("a");
    }

    @Test
    void shouldHandleLongSessionsAndHugeStepCounts() {
        BrowserHistory b = browser(10);
        int t = b.openTab();
        for (int i = 0; i < 100_000; i++) {
            b.visit(t, "https://example.com/" + i);
        }

        assertThat(b.history(100)).hasSize(10);
        assertThat(b.back(t, 99_999)).isEqualTo("https://example.com/0");
        assertThat(b.back(t, Integer.MAX_VALUE)).isEqualTo("about:blank");
        assertThat(b.forward(t, Integer.MAX_VALUE)).isEqualTo("https://example.com/99999");
        assertThat(b.forward(t, 1)).isEqualTo("https://example.com/99999");
    }

    @Test
    void shouldDropTheOldestEntriesQuicklyWhenTheCapacityIsLarge() {
        BrowserHistory b = browser(50_000);
        int t = b.openTab();

        Concurrently.run(1, 20_000, x -> {
            for (int i = 0; i < 300_000; i++) {
                b.visit(t, "u" + i);
            }
        });

        List<String> recent = b.history(3);
        assertThat(recent.get(0)).endsWith("|u299999|1");
        assertThat(b.search("u250000")).containsExactly("u250000");
        assertThat(b.search("u249999")).isEmpty();
        assertThat(b.history(100_000)).hasSize(50_000);
    }

    @Test
    void shouldSurviveTabsBeingClosedWhileOthersBrowse() {
        BrowserHistory b = browser(100_000);

        Concurrently.run(8, 30_000, t -> {
            for (int i = 0; i < 300; i++) {
                int tab = b.openTab();
                b.visit(tab, "https://t" + t + ".com/" + i);
                b.visit(tab, "https://t" + t + ".com/" + i + "/next");
                b.back(tab, 1);
                b.closeTab(tab);
                assertThatThrownBy(() -> b.visit(tab, "x")).isInstanceOf(IllegalArgumentException.class);
            }
        });

        assertThat(b.history(100_000)).hasSize(8 * 300 * 2);
        assertThat(b.openTab()).isEqualTo(8 * 300 + 1);
    }

    @Test
    void shouldKeepEveryReaderViewConsistentWhileWritersAndClearsRun() {
        BrowserHistory b = browser(500);
        int tab = b.openTab();

        Concurrently.run(8, 30_000, t -> {
            for (int i = 0; i < 2_000; i++) {
                if (t < 4) {
                    b.visit(tab, "u" + t + "-" + i);
                } else if (t == 4 && i % 100 == 0) {
                    b.clearHistory(0);
                } else {
                    for (String line : b.history(20)) {
                        assertThat(line.split("\\|")).hasSize(3);
                    }
                    assertThat(b.search("u1-").size()).isLessThanOrEqualTo(500);
                    assertThat(b.mostVisited(5).size()).isLessThanOrEqualTo(5);
                }
            }
        });

        assertThat(b.history(10_000).size()).isLessThanOrEqualTo(500);
        assertThat(b.currentUrl(tab)).startsWith("u");
    }
}

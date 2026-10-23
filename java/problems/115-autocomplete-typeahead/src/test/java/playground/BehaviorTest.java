package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Ranking;
import contract.Typeahead;
import java.util.List;
import java.util.Random;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class BehaviorTest {

    private final FakeClock clock = new FakeClock(1_000_000);

    private Typeahead typeahead() {
        return typeahead(Ranking.FREQUENCY);
    }

    private Typeahead typeahead(Ranking ranking) {
        return Candidate.of(() -> Solution.create(clock, ranking));
    }

    private static void record(Typeahead t, String term, int times) {
        for (int i = 0; i < times; i++) {
            t.record(term);
        }
    }

    @Test
    void shouldRankSuggestionsByFrequencyDescending() {
        Typeahead t = typeahead();
        record(t, "apple", 3);
        record(t, "application", 5);
        record(t, "apply", 1);

        assertThat(t.suggest("app", 10)).containsExactly("application", "apple", "apply");
    }

    @Test
    void shouldBreakFrequencyTiesAlphabetically() {
        Typeahead t = typeahead();
        t.record("pear");
        t.record("peach");
        t.record("pecan");
        t.record("pea");

        assertThat(t.suggest("pe", 10)).containsExactly("pea", "peach", "pear", "pecan");
    }

    @Test
    void shouldReturnOnlyTermsThatStartWithThePrefix() {
        Typeahead t = typeahead();
        t.record("banana", 9);
        t.record("band", 2);
        t.record("abandon", 50);
        t.record("ban");

        assertThat(t.suggest("ban", 10)).containsExactly("banana", "band", "ban");
        assertThat(t.suggest("and", 10)).isEmpty();
    }

    @Test
    void shouldIncludeATermThatEqualsThePrefix() {
        Typeahead t = typeahead();
        t.record("go", 5);
        t.record("golang", 2);

        assertThat(t.suggest("go", 5)).containsExactly("go", "golang");
        assertThat(t.suggest("golang", 5)).containsExactly("golang");
    }

    @Test
    void shouldReturnAtMostKSuggestions() {
        Typeahead t = typeahead();
        for (int i = 0; i < 20; i++) {
            t.record("item" + (char) ('a' + i), 20 - i);
        }

        assertThat(t.suggest("item", 3)).containsExactly("itema", "itemb", "itemc");
        assertThat(t.suggest("item", 1)).containsExactly("itema");
        assertThat(t.suggest("item", 1_000)).hasSize(20);
    }

    @Test
    void shouldReturnAnEmptyListWhenNothingMatches() {
        Typeahead t = typeahead();
        t.record("short");

        assertThat(t.suggest("zzz", 5)).isEmpty();
        assertThat(typeahead().suggest("anything", 5)).isEmpty();
    }

    @Test
    void shouldReturnAnEmptyListWhenThePrefixIsLongerThanEveryTerm() {
        Typeahead t = typeahead();
        t.record("cat", 3);
        t.record("cats", 2);

        assertThat(t.suggest("catsup-and-more", 5)).isEmpty();
        assertThat(t.suggest("cats ", 5)).containsExactly("cats"); // surrounding whitespace is trimmed
    }

    @Test
    void shouldReturnTheTopKOverallForAnEmptyPrefix() {
        Typeahead t = typeahead();
        t.record("zebra", 10);
        t.record("apple", 4);
        t.record("mango", 7);
        t.record("kiwi", 1);

        assertThat(t.suggest("", 2)).containsExactly("zebra", "mango");
        assertThat(t.suggest("   ", 10)).containsExactly("zebra", "mango", "apple", "kiwi");
    }

    @Test
    void shouldIgnoreCaseAndSurroundingWhitespace() {
        Typeahead t = typeahead();
        t.record("  Hello World ");
        t.record("HELLO world");
        t.record("hello");

        assertThat(t.count("hello world")).isEqualTo(2);
        assertThat(t.count("HELLO")).isEqualTo(1);
        assertThat(t.suggest("HeL", 5)).containsExactly("hello world", "hello");
        assertThat(t.suggest(" hello w", 5)).containsExactly("hello world");
    }

    @Test
    void shouldAddABulkCountAndStartAtZero() {
        Typeahead t = typeahead();

        assertThat(t.count("never")).isZero();
        t.record("bulk", 40);
        t.record("bulk");
        t.record("bulk", 9);

        assertThat(t.count("bulk")).isEqualTo(50);
    }

    @Test
    void shouldMakeUpdatesVisibleImmediatelyAndReRank() {
        Typeahead t = typeahead();
        record(t, "red", 5);
        record(t, "reed", 3);
        assertThat(t.suggest("re", 5)).containsExactly("red", "reed");

        t.record("reed", 3);
        assertThat(t.suggest("re", 5)).containsExactly("reed", "red");

        t.record("reed", 100);
        t.record("redwood", 107);
        assertThat(t.suggest("re", 1)).containsExactly("redwood");
    }

    @Test
    void shouldKeepTermsWithSpacesAsOneTerm() {
        Typeahead t = typeahead();
        t.record("new york", 4);
        t.record("new jersey", 6);
        t.record("newark", 5);

        assertThat(t.suggest("new", 5)).containsExactly("new jersey", "newark", "new york");
        assertThat(t.suggest("new y", 5)).containsExactly("new york");
    }

    @Test
    void shouldRejectBlankOrNullTermsAndNonPositiveCounts() {
        Typeahead t = typeahead();
        t.record("ok");

        assertThatThrownBy(() -> t.record(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.record("")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.record("   ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.record("ok", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.record("ok", -3)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.record("fresh", 0)).isInstanceOf(IllegalArgumentException.class);

        assertThat(t.count("ok")).isEqualTo(1);
        assertThat(t.count("fresh")).isZero();
        assertThat(t.suggest("", 10)).containsExactly("ok");
    }

    @Test
    void shouldRejectNonPositiveKAndNullPrefix() {
        Typeahead t = typeahead();
        t.record("ok");

        assertThatThrownBy(() -> t.suggest("o", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.suggest("o", -1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.suggest(null, 3)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.suggestFor("u1", "o", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.suggestFor("u1", null, 3)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.suggestFor(" ", "o", 3)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.suggestFor(null, "o", 3)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.recordFor(" ", "ok")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.recordFor("u1", " ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.setRanking(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.count(" ")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRankByRecencyMostRecentFirst() {
        Typeahead t = typeahead(Ranking.RECENCY);
        t.record("alpha", 100);
        clock.advance(10);
        t.record("alps");
        clock.advance(10);
        t.record("alpine", 5);

        assertThat(t.suggest("al", 10)).containsExactly("alpine", "alps", "alpha");

        clock.advance(10);
        t.record("alpha"); // recorded again: back to the front even though its count was already the largest
        assertThat(t.suggest("al", 10)).containsExactly("alpha", "alpine", "alps");
        assertThat(t.suggest("al", 2)).containsExactly("alpha", "alpine");
    }

    @Test
    void shouldBreakRecencyTiesAlphabetically() {
        Typeahead t = typeahead(Ranking.RECENCY);
        t.record("tea");
        t.record("ten");
        t.record("tab");
        clock.advance(1);
        t.record("tex");

        assertThat(t.suggest("t", 10)).containsExactly("tex", "tab", "tea", "ten");
    }

    @Test
    void shouldSwitchTheRankingRuleAtRuntimeKeepingAllData() {
        Typeahead t = typeahead(Ranking.FREQUENCY);
        t.record("lamp", 9);
        clock.advance(5);
        t.record("lake", 1);
        clock.advance(5);
        t.record("lane", 4);

        assertThat(t.suggest("la", 10)).containsExactly("lamp", "lane", "lake");
        t.setRanking(Ranking.RECENCY);
        assertThat(t.suggest("la", 10)).containsExactly("lane", "lake", "lamp");
        t.setRanking(Ranking.FREQUENCY);
        assertThat(t.suggest("la", 10)).containsExactly("lamp", "lane", "lake");
        assertThat(t.count("lamp")).isEqualTo(9);
    }

    @Test
    void shouldPutTheUsersOwnTermsFirst() {
        Typeahead t = typeahead();
        t.record("java", 50);
        t.record("javascript", 40);
        t.record("jazz", 30);
        t.recordFor("ann", "jazz");
        t.recordFor("ann", "jazz");
        t.recordFor("ann", "javascript");

        assertThat(t.suggest("ja", 10)).containsExactly("java", "javascript", "jazz");
        assertThat(t.suggestFor("ann", "ja", 10)).containsExactly("jazz", "javascript", "java");
        assertThat(t.suggestFor("ann", "ja", 2)).containsExactly("jazz", "javascript");
    }

    @Test
    void shouldIsolateUserHistoriesAndFallBackToGlobalForUnknownUsers() {
        Typeahead t = typeahead();
        t.record("sql", 10);
        t.record("sqlite", 5);
        t.recordFor("ann", "sqlite");
        t.recordFor("bob", "sql");

        assertThat(t.suggestFor("ann", "sql", 5)).containsExactly("sqlite", "sql");
        assertThat(t.suggestFor("bob", "sql", 5)).containsExactly("sql", "sqlite");
        assertThat(t.suggestFor("nobody", "sql", 5)).isEqualTo(t.suggest("sql", 5));
    }

    @Test
    void shouldCountPersonalRecordsInTheGlobalCountsToo() {
        Typeahead t = typeahead();
        t.recordFor("ann", "kotlin");
        t.recordFor("bob", "kotlin");
        t.record("kotlin");
        t.record("kotor", 2);

        assertThat(t.count("kotlin")).isEqualTo(3);
        assertThat(t.suggest("kot", 5)).containsExactly("kotlin", "kotor");
    }

    @Test
    void shouldOrderTermsWithEqualUserCountsByTheActiveRanking() {
        Typeahead t = typeahead(Ranking.FREQUENCY);
        t.record("rust", 5);
        t.record("ruby", 9);
        t.record("rune", 7);
        t.recordFor("ann", "rust"); // rust now 6 globally, ann: rust=1

        assertThat(t.suggestFor("ann", "ru", 5)).containsExactly("rust", "ruby", "rune");

        t.setRanking(Ranking.RECENCY);
        clock.advance(3);
        t.record("rune");
        assertThat(t.suggestFor("ann", "ru", 5)).containsExactly("rust", "rune", "ruby");
    }

    @Test
    void shouldHaveExactCountsWhenManyThreadsRecordConcurrently() {
        Typeahead t = typeahead();

        Concurrently.run(16, id -> {
            for (int i = 0; i < 5_000; i++) {
                t.record("shared");
                t.record("term" + (i % 10));
                t.record("bulk", 2);
            }
        });

        assertThat(t.count("shared")).isEqualTo(80_000);
        assertThat(t.count("bulk")).isEqualTo(160_000);
        for (int i = 0; i < 10; i++) {
            assertThat(t.count("term" + i)).isEqualTo(8_000);
        }
        assertThat(t.suggest("", 3)).containsExactly("bulk", "shared", "term0");
    }

    @Test
    void shouldServeReadersWhileWritersUpdate() {
        Typeahead t = typeahead();
        for (int i = 0; i < 100; i++) {
            t.record("seed" + i);
        }
        AtomicInteger bad = new AtomicInteger();

        Concurrently.run(8, id -> {
            for (int i = 0; i < 3_000; i++) {
                if (id < 4) {
                    t.recordFor("u" + id, "seed" + (i % 100));
                    t.record("w" + id + "-" + (i % 50));
                } else {
                    List<String> r = t.suggest("seed", 5);
                    if (r.size() != 5 || r.stream().distinct().count() != 5 || !r.stream().allMatch(s -> s.startsWith("seed"))) {
                        bad.incrementAndGet();
                    }
                    t.suggestFor("u" + (id - 4), "w", 3);
                }
            }
        });

        assertThat(bad).hasValue(0);
        long total = 0;
        for (int i = 0; i < 100; i++) {
            total += t.count("seed" + i);
        }
        assertThat(total).isEqualTo(100 + 4 * 3_000);
    }

    @Test
    void shouldHandleOneHundredThousandTermsAndTenThousandSuggestCalls() {
        Typeahead t = typeahead();
        Random rnd = new Random(42);
        String[] terms = new String[100_000];
        for (int i = 0; i < terms.length; i++) {
            StringBuilder sb = new StringBuilder();
            int len = 4 + rnd.nextInt(8);
            for (int c = 0; c < len; c++) {
                sb.append((char) ('a' + rnd.nextInt(26)));
            }
            terms[i] = sb.toString() + i;
        }
        for (int i = 0; i < terms.length; i++) {
            t.record(terms[i], 1 + (i % 97));
        }

        long start = System.nanoTime();
        int nonEmpty = 0;
        for (int q = 0; q < 10_000; q++) {
            String term = terms[(q * 7) % terms.length];
            List<String> r = t.suggest(term.substring(0, 1 + (q % 3)), 10);
            if (!r.isEmpty()) {
                nonEmpty++;
            }
            assertThat(r.size()).isLessThanOrEqualTo(10);
        }
        long elapsedMs = (System.nanoTime() - start) / 1_000_000;

        assertThat(nonEmpty).isEqualTo(10_000);
        assertThat(elapsedMs).as("10,000 suggest calls over 100,000 terms").isLessThan(30_000);
        assertThat(t.suggest("", 1)).hasSize(1);
    }
}

package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.Ranking;
import contract.Typeahead;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Random;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;
import testkit.FakeClock;

class HiddenScenariosTest {

    private final FakeClock clock = new FakeClock(5_000);

    private Typeahead typeahead(Ranking r) {
        return Candidate.of(() -> Solution.create(clock, r));
    }

    @Test
    void shouldNotKeepAStaleTopKWhenALowRankedTermOvertakesTheLeaders() {
        Typeahead t = typeahead(Ranking.FREQUENCY);
        for (int i = 0; i < 30; i++) {
            t.record("ab" + (char) ('a' + i % 26) + i, 100 - i);
        }
        t.record("abzzz-late", 1);
        assertThat(t.suggest("ab", 3)).doesNotContain("abzzz-late");

        t.record("abzzz-late", 500);
        assertThat(t.suggest("ab", 3)).startsWith("abzzz-late");
        assertThat(t.suggest("abz", 3)).startsWith("abzzz-late");
        assertThat(t.suggest("", 1)).containsExactly("abzzz-late");
        assertThat(t.suggest("abzzz-l", 5)).containsExactly("abzzz-late");
    }

    @Test
    void shouldBreakAThousandWayTieAlphabeticallyForAnyK() {
        Typeahead t = typeahead(Ranking.FREQUENCY);
        List<String> terms = new ArrayList<>();
        for (int i = 0; i < 1_000; i++) {
            terms.add(String.format("tie%04d", (i * 7919) % 1_000));
        }
        for (String s : terms) {
            t.record(s, 3);
        }

        assertThat(t.suggest("tie", 4)).containsExactly("tie0000", "tie0001", "tie0002", "tie0003");
        assertThat(t.suggest("tie09", 100)).hasSize(100).isSorted();
        assertThat(t.suggest("", 1_000)).hasSize(1_000).isSorted();
    }

    @Test
    void shouldReRankByRecencyOnBulkAndPersonalRecordsToo() {
        Typeahead t = typeahead(Ranking.RECENCY);
        t.record("moon", 1_000);
        clock.advance(1);
        t.record("moss");
        clock.advance(1);
        t.recordFor("ann", "mole");
        clock.advance(1);
        t.record("moon", 2);

        assertThat(t.suggest("mo", 10)).containsExactly("moon", "mole", "moss");
        assertThat(t.suggestFor("ann", "mo", 10)).containsExactly("mole", "moon", "moss");
        t.setRanking(Ranking.FREQUENCY);
        assertThat(t.suggest("mo", 10)).containsExactly("moon", "mole", "moss");
    }

    @Test
    void shouldLeaveNoTraceOfRejectedCalls() {
        Typeahead t = typeahead(Ranking.FREQUENCY);
        t.record("kept", 2);
        try {
            t.record("ghost", 0);
        } catch (IllegalArgumentException expected) {
            // fine
        }
        try {
            t.recordFor(" ", "ghost");
        } catch (IllegalArgumentException expected) {
            // fine
        }
        try {
            t.recordFor("ann", "  ");
        } catch (IllegalArgumentException expected) {
            // fine
        }

        assertThat(t.count("ghost")).isZero();
        assertThat(t.suggest("", 10)).containsExactly("kept");
        assertThat(t.suggestFor("ann", "", 10)).containsExactly("kept");
    }

    @Test
    void shouldAgreeWithABruteForceModelOnRandomOperations() {
        Typeahead t = typeahead(Ranking.FREQUENCY);
        Map<String, Long> model = new HashMap<>();
        Random rnd = new Random(7);
        String[] alphabet = {"a", "b", "c"};
        for (int step = 0; step < 6_000; step++) {
            StringBuilder sb = new StringBuilder();
            int len = 1 + rnd.nextInt(5);
            for (int i = 0; i < len; i++) {
                sb.append(alphabet[rnd.nextInt(3)]);
            }
            String term = sb.toString();
            long c = 1 + rnd.nextInt(4);
            t.record(rnd.nextBoolean() ? term.toUpperCase() : term, c);
            model.merge(term, c, Long::sum);

            if (step % 10 == 0) {
                StringBuilder pb = new StringBuilder();
                for (int i = rnd.nextInt(4); i > 0; i--) {
                    pb.append(alphabet[rnd.nextInt(3)]);
                }
                String prefix = pb.toString();
                int k = 1 + rnd.nextInt(8);
                List<String> expected = model.entrySet().stream()
                        .filter(e -> e.getKey().startsWith(prefix))
                        .sorted(Map.Entry.<String, Long>comparingByValue().reversed().thenComparing(Map.Entry.comparingByKey()))
                        .limit(k)
                        .map(Map.Entry::getKey)
                        .toList();
                assertThat(t.suggest(prefix, k)).as("prefix '%s' k=%d step %d", prefix, k, step).isEqualTo(expected);
            }
        }
    }

    @Test
    void shouldKeepAHotTermExactWhileReadersAskForItsPrefix() {
        Typeahead t = typeahead(Ranking.FREQUENCY);
        for (int i = 0; i < 200; i++) {
            t.record("hot" + i, 10);
        }
        AtomicInteger violations = new AtomicInteger();

        Concurrently.run(12, 60_000, id -> {
            for (int i = 0; i < 20_000; i++) {
                if (id < 8) {
                    t.record("hot" + (i % 200));
                    t.record("hot0");
                } else {
                    List<String> r = t.suggest("hot", 5);
                    if (r.size() != 5 || r.stream().distinct().count() != 5) {
                        violations.incrementAndGet();
                    }
                }
            }
        });

        assertThat(violations).hasValue(0);
        assertThat(t.count("hot0")).isEqualTo(10 + 8 * 100 + 8 * 20_000);
        assertThat(t.count("hot199")).isEqualTo(10 + 8 * 100);
        assertThat(t.suggest("hot", 1)).containsExactly("hot0");
        // all the other 199 terms have exactly the same count: alphabetical
        assertThat(t.suggest("hot", 4)).containsExactly("hot0", "hot1", "hot10", "hot100");
    }

    @Test
    void shouldKeepPerUserHistoriesExactUnderConcurrency() {
        Typeahead t = typeahead(Ranking.FREQUENCY);
        t.record("zeta", 1_000_000);
        t.record("zeal", 1_000_000);

        Concurrently.run(8, 60_000, id -> {
            for (int i = 0; i < 2_000; i++) {
                t.recordFor("user" + (id % 2), id % 2 == 0 ? "zeta" : "zeal");
                t.suggestFor("user" + (id % 2), "ze", 2);
            }
        });

        assertThat(t.count("zeta")).isEqualTo(1_000_000 + 4 * 2_000);
        assertThat(t.suggestFor("user0", "ze", 2)).containsExactly("zeta", "zeal");
        assertThat(t.suggestFor("user1", "ze", 2)).containsExactly("zeal", "zeta");
        assertThat(t.suggestFor("user2", "ze", 2)).containsExactly("zeal", "zeta"); // equal globals: alphabetical
    }

    @Test
    void shouldHandleLongTermsAndANonAsciiAlphabet() {
        Typeahead t = typeahead(Ranking.FREQUENCY);
        String longTerm = "x".repeat(2_000);
        t.record(longTerm, 3);
        t.record(longTerm + "y", 2);
        t.record("ÉCOLE", 4);
        t.record("école normale", 6);

        assertThat(t.suggest("x".repeat(1_500), 5)).containsExactly(longTerm, longTerm + "y");
        assertThat(t.suggest(longTerm + "yy", 5)).isEmpty();
        assertThat(t.suggest("Éc", 5)).containsExactly("école normale", "école");
    }

    @Test
    void shouldServeTwoHundredThousandTermsAndTwentyThousandQueriesQuickly() {
        Typeahead t = typeahead(Ranking.FREQUENCY);
        for (int i = 0; i < 200_000; i++) {
            t.record("q" + Integer.toString(i * 31, 36), 1 + (i % 50));
        }

        long start = System.nanoTime();
        for (int i = 0; i < 20_000; i++) {
            String s = Integer.toString(i * 31, 36);
            t.suggest("q" + s.substring(0, Math.min(s.length(), 1 + i % 2)), 10);
        }
        long ms = (System.nanoTime() - start) / 1_000_000;

        assertThat(ms).as("20,000 suggest calls over 200,000 terms").isLessThan(60_000);
        assertThat(t.suggest("q", 10)).hasSize(10);
    }

    @Test
    void shouldReturnAStableOrderWhenCountsAreRaisedToTheSameValue() {
        Typeahead t = typeahead(Ranking.FREQUENCY);
        t.record("beta", 5);
        t.record("alpha", 3);
        t.record("alpha", 2);

        assertThat(t.suggest("", 5)).containsExactly("alpha", "beta");
        t.record("beta");
        assertThat(t.suggest("", 5)).containsExactly("beta", "alpha");
    }
}

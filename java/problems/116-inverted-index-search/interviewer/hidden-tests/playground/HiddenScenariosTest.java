package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.within;
import static org.junit.jupiter.api.Assertions.assertTimeoutPreemptively;

import contract.SearchEngine;
import java.time.Duration;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.LinkedHashSet;
import java.util.List;
import java.util.Map;
import java.util.Random;
import java.util.Set;
import java.util.TreeMap;
import java.util.concurrent.atomic.AtomicBoolean;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    private static final double EPS = 1e-9;

    private static double idf(int n, int df) {
        return Math.log(1.0 + (double) n / df);
    }

    @Test
    void shouldShiftIdfAsUnrelatedDocumentsAreAdded() {
        SearchEngine e = Candidate.of(Solution::create);
        e.addDocument("a", "rare words here");
        assertThat(e.scoreOf("rare", "a")).isCloseTo(1.0 / 3 * idf(1, 1), within(EPS));

        for (int i = 0; i < 9; i++) {
            e.addDocument("filler" + i, "nothing relevant");
        }

        assertThat(e.scoreOf("rare", "a")).isCloseTo(1.0 / 3 * idf(10, 1), within(EPS));
    }

    @Test
    void shouldFlipTheRankingWhenAnUpdateChangesTermFrequencies() {
        SearchEngine e = Candidate.of(Solution::create);
        e.addDocument("a", "cat dog");
        e.addDocument("b", "cat dog bird fish");
        e.addDocument("c", "bird");
        assertThat(e.search("cat", "OR", 5)).containsExactly("a", "b");

        e.updateDocument("a", "x y z w cat");
        e.updateDocument("b", "cat");

        assertThat(e.search("cat", "OR", 5)).containsExactly("b", "a");
        assertThat(e.search("dog", "OR", 5)).isEmpty();
        assertThat(e.search("bird", "OR", 5)).containsExactly("c");
        assertThat(e.documentCount()).isEqualTo(3);
    }

    @Test
    void shouldWorkFromAnEmptyCollectionAndAfterEverythingWasDeleted() {
        SearchEngine e = Candidate.of(Solution::create);
        assertThat(e.search("anything", "OR", 5)).isEmpty();
        assertThat(e.documentCount()).isZero();

        e.addDocument("a", "one two");
        e.addDocument("b", "two three");
        e.deleteDocument("a");
        e.deleteDocument("b");

        assertThat(e.documentCount()).isZero();
        assertThat(e.search("two", "OR", 5)).isEmpty();
        e.addDocument("a", "two");
        assertThat(e.search("two", "AND", 5)).containsExactly("a");
        assertThat(e.scoreOf("two", "a")).isCloseTo(idf(1, 1), within(EPS));
    }

    @Test
    void shouldTreatTokensWithDigitsAndMixedCaseAsSingleTerms() {
        SearchEngine e = Candidate.of(Solution::create);
        e.addDocument("a", "Model X100 vs model-X200");
        e.addDocument("b", "x100x200");

        assertThat(e.search("x100", "OR", 5)).containsExactly("a");
        assertThat(e.search("X200 MODEL", "AND", 5)).containsExactly("a");
        assertThat(e.search("x100x200", "AND", 5)).containsExactly("b");
        assertThat(e.scoreOf("model", "a")).isCloseTo(2.0 / 5 * idf(2, 1), within(EPS));
    }

    @Test
    void shouldMatchABruteForceOracleAfterManyRandomOperations() {
        SearchEngine e = Candidate.of(Solution::create);
        Map<String, String> oracle = new TreeMap<>();
        Random rnd = new Random(42);
        String[] vocab = {"red", "green", "blue", "cat", "dog", "fish", "sun", "moon", "star", "tree", "rock", "sand"};

        for (int step = 0; step < 1_500; step++) {
            String id = "doc" + rnd.nextInt(120);
            int op = rnd.nextInt(10);
            String text = randomText(rnd, vocab);
            if (op < 5) {
                if (!oracle.containsKey(id)) {
                    e.addDocument(id, text);
                    oracle.put(id, text);
                }
            } else if (op < 7) {
                if (oracle.containsKey(id)) {
                    e.updateDocument(id, text);
                    oracle.put(id, text);
                }
            } else if (oracle.containsKey(id)) {
                e.deleteDocument(id);
                oracle.remove(id);
            }
            if (step % 25 == 0) {
                String query = randomText(rnd, vocab).replace("  ", " ");
                for (String mode : new String[] {"AND", "OR"}) {
                    assertThat(e.search(query, mode, 1_000))
                            .as("step %d query '%s' %s", step, query, mode)
                            .isEqualTo(rank(oracle, query, mode));
                }
            }
        }
        assertThat(e.documentCount()).isEqualTo(oracle.size());
        for (String id : oracle.keySet()) {
            assertThat(e.scoreOf("red cat star", id)).isCloseTo(score(oracle, "red cat star", id), within(EPS));
        }
    }

    private static String randomText(Random rnd, String[] vocab) {
        int n = rnd.nextInt(7);
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < n; i++) {
            sb.append(vocab[rnd.nextInt(vocab.length)]).append(i % 2 == 0 ? " " : ", ");
        }
        return sb.toString();
    }

    private static List<String> tokens(String s) {
        List<String> out = new ArrayList<>();
        for (String t : s.toLowerCase().split("[^a-z0-9]+")) {
            if (!t.isEmpty()) {
                out.add(t);
            }
        }
        return out;
    }

    private static double score(Map<String, String> docs, String query, String id) {
        Set<String> terms = new LinkedHashSet<>(tokens(query));
        List<String> doc = tokens(docs.get(id));
        double s = 0;
        for (String t : terms) {
            long tf = doc.stream().filter(t::equals).count();
            if (tf == 0) {
                continue;
            }
            long df = docs.values().stream().filter(d -> tokens(d).contains(t)).count();
            s += ((double) tf / doc.size()) * idf(docs.size(), (int) df);
        }
        return s;
    }

    private static List<String> rank(Map<String, String> docs, String query, String mode) {
        Set<String> terms = new LinkedHashSet<>(tokens(query));
        Map<String, Double> scores = new HashMap<>();
        List<String> ids = new ArrayList<>();
        if (terms.isEmpty()) {
            return ids;
        }
        for (String id : docs.keySet()) {
            List<String> doc = tokens(docs.get(id));
            boolean ok = mode.equals("AND") ? doc.containsAll(terms) : terms.stream().anyMatch(doc::contains);
            if (ok) {
                ids.add(id);
                scores.put(id, score(docs, query, id));
            }
        }
        ids.sort((a, b) -> {
            int c = Double.compare(scores.get(b), scores.get(a));
            return c != 0 ? c : a.compareTo(b);
        });
        return ids;
    }

    @Test
    void shouldNeverShowAHalfAppliedUpdateToConcurrentSearches() {
        SearchEngine e = Candidate.of(Solution::create);
        e.addDocument("flip", "alpha beta");
        e.addDocument("other", "gamma delta");
        AtomicBoolean done = new AtomicBoolean();

        Concurrently.run(5, 60_000, t -> {
            if (t == 0) {
                for (int i = 0; i < 20_000; i++) {
                    e.updateDocument("flip", i % 2 == 0 ? "gamma delta" : "alpha beta");
                }
                done.set(true);
                return;
            }
            while (!done.get()) {
                // the document is always exactly one of the two versions; never a mixture, never absent
                assertThat(e.search("alpha delta", "AND", 5)).isEmpty();
                assertThat(e.search("alpha gamma", "OR", 5)).containsExactlyInAnyOrder("flip", "other");
                assertThat(e.documentCount()).isEqualTo(2);
            }
        });
    }

    @Test
    void shouldStayFastOnALargeCollectionWithSelectiveQueries() {
        SearchEngine e = Candidate.of(Solution::create);
        for (int i = 0; i < 30_000; i++) {
            e.addDocument("d" + i, "common filler words w" + (i % 3_000) + " more text here");
        }

        assertTimeoutPreemptively(Duration.ofSeconds(30), () -> {
            for (int q = 0; q < 20_000; q++) {
                List<String> r = e.search("w" + (q % 3_000) + " nothing", "OR", 5);
                assertThat(r).hasSize(5);
            }
            assertThat(e.search("common w7", "AND", 100)).hasSize(10);
        });
    }

    @Test
    void shouldPageConsistentlyThroughAShrinkingCollection() {
        SearchEngine e = Candidate.of(Solution::create);
        for (int i = 0; i < 20; i++) {
            e.addDocument(String.format("p%02d", i), "topic");
        }
        List<String> first = e.search("topic", "OR", 0, 5);
        assertThat(first).containsExactly("p00", "p01", "p02", "p03", "p04");

        e.deleteDocument("p00");
        e.deleteDocument("p07");

        assertThat(e.search("topic", "OR", 5, 5)).containsExactly("p06", "p08", "p09", "p10", "p11");
        assertThat(e.search("topic", "OR", 16, 5)).containsExactly("p18", "p19");
    }
}

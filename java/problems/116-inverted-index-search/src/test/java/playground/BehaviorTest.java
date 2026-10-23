package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;
import static org.assertj.core.api.Assertions.within;

import contract.SearchEngine;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private static final double EPS = 1e-9;

    private SearchEngine engine() {
        return Candidate.of(Solution::create);
    }

    private static double idf(int n, int df) {
        return Math.log(1.0 + (double) n / df);
    }

    /** d1 "the quick brown fox", d2 "the lazy dog", d3 "quick quick fox". */
    private SearchEngine corpus() {
        SearchEngine e = engine();
        e.addDocument("d1", "the quick brown fox");
        e.addDocument("d2", "the lazy dog");
        e.addDocument("d3", "quick quick fox");
        return e;
    }

    @Test
    void shouldFindDocumentsContainingASingleTerm() {
        SearchEngine e = corpus();

        assertThat(e.search("lazy", "OR", 10)).containsExactly("d2");
        assertThat(e.search("lazy", "AND", 10)).containsExactly("d2");
        assertThat(e.search("fox", "OR", 10)).containsExactlyInAnyOrder("d1", "d3");
        assertThat(e.documentCount()).isEqualTo(3);
    }

    @Test
    void shouldTokeniseCaseInsensitivelyOnNonAlphanumerics() {
        SearchEngine e = engine();
        e.addDocument("a", "Hello, WORLD!  Foo_bar-baz;qux2024...");

        assertThat(e.search("hello", "AND", 5)).containsExactly("a");
        assertThat(e.search("WoRlD", "AND", 5)).containsExactly("a");
        assertThat(e.search("foo bar baz", "AND", 5)).containsExactly("a");
        assertThat(e.search("qux2024", "AND", 5)).containsExactly("a");
        assertThat(e.search("qux", "AND", 5)).isEmpty();
        assertThat(e.search("foo_bar", "AND", 5)).containsExactly("a");
        assertThat(e.search("helloworld", "OR", 5)).isEmpty();
    }

    @Test
    void shouldScoreWithTheDocumentedTfIdfFormula() {
        SearchEngine e = corpus();

        // N = 3, df(fox) = 2, df(quick) = 2, df(the) = 2, df(lazy) = 1
        assertThat(e.scoreOf("fox", "d1")).isCloseTo(0.25 * idf(3, 2), within(EPS));
        assertThat(e.scoreOf("fox", "d3")).isCloseTo(1.0 / 3 * idf(3, 2), within(EPS));
        assertThat(e.scoreOf("quick", "d3")).isCloseTo(2.0 / 3 * idf(3, 2), within(EPS));
        assertThat(e.scoreOf("lazy", "d2")).isCloseTo(1.0 / 3 * idf(3, 1), within(EPS));
        assertThat(e.scoreOf("quick fox", "d3")).isCloseTo(idf(3, 2), within(EPS));
        assertThat(e.scoreOf("quick fox", "d1")).isCloseTo(0.5 * idf(3, 2), within(EPS));
    }

    @Test
    void shouldRankByScoreDescending() {
        SearchEngine e = corpus();

        assertThat(e.search("quick fox", "OR", 10)).containsExactly("d3", "d1");
        assertThat(e.search("the", "OR", 10)).containsExactly("d2", "d1");
        assertThat(e.search("lazy fox", "OR", 10)).containsExactly("d2", "d3", "d1");
    }

    @Test
    void shouldNormaliseTermFrequencyByDocumentLength() {
        SearchEngine e = engine();
        e.addDocument("short", "apple pie");
        e.addDocument("long", "apple pie with a lot of other words around it");
        e.addDocument("other", "banana");

        assertThat(e.search("apple", "OR", 10)).containsExactly("short", "long");
        assertThat(e.scoreOf("apple", "short")).isCloseTo(0.5 * idf(3, 2), within(EPS));
        assertThat(e.scoreOf("apple", "long")).isCloseTo(1.0 / 10 * idf(3, 2), within(EPS));
    }

    @Test
    void shouldRequireEveryTermInAndMode() {
        SearchEngine e = corpus();

        assertThat(e.search("quick fox", "AND", 10)).containsExactly("d3", "d1");
        assertThat(e.search("quick brown", "AND", 10)).containsExactly("d1");
        assertThat(e.search("lazy fox", "AND", 10)).isEmpty();
    }

    @Test
    void shouldMatchAnyTermInOrMode() {
        SearchEngine e = corpus();

        assertThat(e.search("brown dog", "OR", 10)).containsExactlyInAnyOrder("d1", "d2");
        assertThat(e.search("brown dog", "OR", 10)).hasSize(2);
    }

    @Test
    void shouldReturnNothingForAnAndQueryWithAnUnknownTerm() {
        SearchEngine e = corpus();

        assertThat(e.search("fox zebra", "AND", 10)).isEmpty();
        assertThat(e.search("zebra", "AND", 10)).isEmpty();
        assertThat(e.search("zebra", "OR", 10)).isEmpty();
    }

    @Test
    void shouldLetUnknownTermsContributeNothingInOrMode() {
        SearchEngine e = corpus();

        assertThat(e.search("fox zebra", "OR", 10)).isEqualTo(e.search("fox", "OR", 10));
        assertThat(e.scoreOf("fox zebra", "d1")).isCloseTo(e.scoreOf("fox", "d1"), within(EPS));
    }

    @Test
    void shouldBreakScoreTiesByDocIdAscending() {
        SearchEngine e = engine();
        e.addDocument("c", "red blue");
        e.addDocument("a", "red blue");
        e.addDocument("b", "blue red");
        e.addDocument("z", "green");
        e.addDocument("m", "red blue green green green");

        assertThat(e.search("red", "OR", 10)).containsExactly("a", "b", "c", "m");
        assertThat(e.search("red blue", "AND", 10)).containsExactly("a", "b", "c", "m");
        assertThat(e.search("red", "OR", 2)).containsExactly("a", "b");
    }

    @Test
    void shouldReturnAtMostKResults() {
        SearchEngine e = engine();
        for (int i = 0; i < 10; i++) {
            e.addDocument("doc" + i, "common word" + i);
        }

        assertThat(e.search("common", "OR", 3)).containsExactly("doc0", "doc1", "doc2");
        assertThat(e.search("common", "OR", 100)).hasSize(10);
        assertThat(e.search("common", "OR", 0)).isEmpty();
    }

    @Test
    void shouldPaginateTheRankingWithOffsetAndLimit() {
        SearchEngine e = engine();
        for (int i = 0; i < 10; i++) {
            e.addDocument(String.format("doc%02d", i), "common " + "pad ".repeat(i));
        }
        List<String> full = e.search("common", "OR", 100);
        // higher tf for shorter documents: doc00 > doc01 > ...
        assertThat(full).hasSize(10).isSorted();

        List<String> glued = new ArrayList<>();
        for (int offset = 0; offset < 10; offset += 4) {
            glued.addAll(e.search("common", "OR", offset, 4));
        }
        assertThat(glued).isEqualTo(full);
        assertThat(e.search("common", "OR", 4, 3)).containsExactlyElementsOf(full.subList(4, 7));
        assertThat(e.search("common", "OR", 8, 5)).containsExactlyElementsOf(full.subList(8, 10));
        assertThat(e.search("common", "OR", 10, 5)).isEmpty();
        assertThat(e.search("common", "OR", 99, 5)).isEmpty();
        assertThat(e.search("common", "OR", 2, 0)).isEmpty();
    }

    @Test
    void shouldRejectAddingAnExistingDocumentId() {
        SearchEngine e = corpus();

        assertThatThrownBy(() -> e.addDocument("d1", "something else"))
                .isInstanceOf(IllegalArgumentException.class);
        assertThat(e.documentCount()).isEqualTo(3);
        assertThat(e.search("something", "OR", 5)).isEmpty();
        assertThat(e.search("brown", "OR", 5)).containsExactly("d1");
    }

    @Test
    void shouldRejectInvalidArguments() {
        SearchEngine e = corpus();

        assertThatThrownBy(() -> e.addDocument(null, "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.addDocument(" ", "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.addDocument("", "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.addDocument("n", null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.search(null, "OR", 5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.search("fox", "XOR", 5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.search("fox", null, 5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.search("fox", "and", 5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.search("fox", "OR", -1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.search("fox", "OR", -1, 5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.search("fox", "OR", 0, -5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.scoreOf(null, "d1")).isInstanceOf(IllegalArgumentException.class);
        assertThat(e.documentCount()).isEqualTo(3);
    }

    @Test
    void shouldReturnNothingForAQueryWithoutTokens() {
        SearchEngine e = corpus();

        assertThat(e.search("", "OR", 10)).isEmpty();
        assertThat(e.search("  ,.! ", "AND", 10)).isEmpty();
        assertThat(e.search("", "AND", 10)).isEmpty();
        assertThat(e.scoreOf("", "d1")).isEqualTo(0.0);
    }

    @Test
    void shouldTreatRepeatedQueryTermsAsASingleTerm() {
        SearchEngine e = corpus();

        assertThat(e.scoreOf("fox fox FOX", "d1")).isCloseTo(e.scoreOf("fox", "d1"), within(EPS));
        assertThat(e.search("fox fox", "AND", 10)).isEqualTo(e.search("fox", "AND", 10));
        assertThat(e.search("quick quick fox", "OR", 10)).isEqualTo(e.search("quick fox", "OR", 10));
    }

    @Test
    void shouldStopReturningADeletedDocument() {
        SearchEngine e = corpus();

        e.deleteDocument("d2");

        assertThat(e.search("lazy", "OR", 10)).isEmpty();
        assertThat(e.search("the", "OR", 10)).containsExactly("d1");
        assertThat(e.documentCount()).isEqualTo(2);
        assertThatThrownBy(() -> e.scoreOf("lazy", "d2")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRecomputeIdfWithTheNewDocumentCountAfterADelete() {
        SearchEngine e = corpus();
        assertThat(e.scoreOf("fox", "d1")).isCloseTo(0.25 * idf(3, 2), within(EPS));

        e.deleteDocument("d2"); // N = 2, df(fox) = 2

        assertThat(e.scoreOf("fox", "d1")).isCloseTo(0.25 * idf(2, 2), within(EPS));
        e.deleteDocument("d3"); // N = 1, df(fox) = 1

        assertThat(e.scoreOf("fox", "d1")).isCloseTo(0.25 * idf(1, 1), within(EPS));
    }

    @Test
    void shouldReflectAnUpdatedTextImmediately() {
        SearchEngine e = corpus();

        e.updateDocument("d2", "quick quick quick dog");

        assertThat(e.search("lazy", "OR", 10)).isEmpty();
        assertThat(e.search("quick", "OR", 10)).containsExactly("d2", "d3", "d1");
        assertThat(e.search("the", "OR", 10)).containsExactly("d1");
        assertThat(e.documentCount()).isEqualTo(3);
        // N = 3, df(quick) = 3
        assertThat(e.scoreOf("quick", "d2")).isCloseTo(0.75 * idf(3, 3), within(EPS));
        assertThat(e.scoreOf("quick", "d1")).isCloseTo(0.25 * idf(3, 3), within(EPS));
    }

    @Test
    void shouldRejectDeletingOrUpdatingAnUnknownDocument() {
        SearchEngine e = corpus();

        assertThatThrownBy(() -> e.deleteDocument("nope")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.updateDocument("nope", "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.updateDocument("d1", null)).isInstanceOf(IllegalArgumentException.class);
        e.deleteDocument("d1");
        assertThatThrownBy(() -> e.deleteDocument("d1")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.updateDocument("d1", "back")).isInstanceOf(IllegalArgumentException.class);
        assertThat(e.documentCount()).isEqualTo(2);
    }

    @Test
    void shouldAllowAddingADeletedIdAgainWithFreshContent() {
        SearchEngine e = corpus();

        e.deleteDocument("d1");
        e.addDocument("d1", "completely different");

        assertThat(e.search("brown", "OR", 10)).isEmpty();
        assertThat(e.search("different", "OR", 10)).containsExactly("d1");
    }

    @Test
    void shouldCountDocumentsWithoutTokensTowardsTheCollectionSize() {
        SearchEngine e = engine();
        e.addDocument("empty", "");
        e.addDocument("punct", "?! ... --");
        e.addDocument("a", "alpha beta");

        assertThat(e.documentCount()).isEqualTo(3);
        assertThat(e.scoreOf("alpha", "a")).isCloseTo(0.5 * idf(3, 1), within(EPS));
        assertThat(e.scoreOf("alpha", "empty")).isEqualTo(0.0);
        assertThat(e.search("alpha", "OR", 10)).containsExactly("a");
        e.updateDocument("empty", "alpha");
        assertThat(e.search("alpha", "OR", 10)).containsExactly("empty", "a");
    }

    @Test
    void shouldScoreAnExistingDocumentWithoutQueryTermsAsZero() {
        SearchEngine e = corpus();

        assertThat(e.scoreOf("lazy", "d1")).isEqualTo(0.0);
        assertThatThrownBy(() -> e.scoreOf("lazy", "ghost")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldNotLoseDocumentsAddedConcurrently() {
        SearchEngine e = engine();

        Concurrently.run(16, t -> {
            for (int i = 0; i < 200; i++) {
                e.addDocument("t" + t + "-" + i, "shared token" + t + " n" + i);
            }
        });

        assertThat(e.documentCount()).isEqualTo(16 * 200);
        assertThat(e.search("shared", "OR", 10_000)).hasSize(3_200);
        assertThat(e.search("token3 n7", "AND", 10)).containsExactly("t3-7");
    }

    @Test
    void shouldAcceptExactlyOneOfManyConcurrentAddsWithTheSameId() {
        SearchEngine e = engine();
        AtomicInteger accepted = new AtomicInteger();

        Concurrently.run(12, t -> {
            try {
                e.addDocument("same", "payload from " + t);
                accepted.incrementAndGet();
            } catch (IllegalArgumentException expected) {
                // duplicate
            }
        });

        assertThat(accepted.get()).isEqualTo(1);
        assertThat(e.documentCount()).isEqualTo(1);
        assertThat(e.search("payload", "OR", 5)).containsExactly("same");
    }

    @Test
    void shouldAnswerSearchesWhileDocumentsAreAddedAndDeleted() {
        SearchEngine e = engine();
        for (int i = 0; i < 100; i++) {
            e.addDocument("base" + i, "stable content " + i);
        }

        Concurrently.run(8, 60_000, t -> {
            for (int i = 0; i < 300; i++) {
                if (t < 4) {
                    String id = "tmp" + t + "-" + i;
                    e.addDocument(id, "volatile content");
                    e.deleteDocument(id);
                } else {
                    List<String> r = e.search("stable content", "AND", 1_000);
                    assertThat(r).hasSize(100);
                }
            }
        });

        assertThat(e.documentCount()).isEqualTo(100);
        assertThat(e.search("volatile", "OR", 10)).isEmpty();
    }
}

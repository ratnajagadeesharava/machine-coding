package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.RagEngine;
import java.util.ArrayList;
import java.util.List;
import java.util.TreeSet;
import org.junit.jupiter.api.Test;
import playground.Fakes.BruteForceStore;
import playground.Fakes.LetterEmbedder;
import playground.Fakes.VocabEmbedder;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private static final String HEAD = "Answer using only the context.\n\nContext:\n";

    private final VocabEmbedder embedder = new VocabEmbedder();

    private RagEngine engine() {
        return Candidate.of(() -> Solution.create(embedder));
    }

    private static String prompt(String context, String query) {
        return HEAD + context + "\n\nQuestion: " + query + "\nAnswer:";
    }

    /** a="alpha beta", b="alpha gamma delta", c="alpha": for the query "alpha" the ranking is c, a, b. */
    private static void ingestPromptCorpus(RagEngine e) {
        e.ingest("a", "alpha beta");
        e.ingest("b", "alpha gamma delta");
        e.ingest("c", "alpha");
    }

    @Test
    void shouldCutTextIntoChunksOfAtMostChunkSizeWords() {
        RagEngine e = engine();
        e.setChunking(4, 0);

        assertThat(e.ingest("doc", Fakes.words("w", 10))).isEqualTo(3);

        assertThat(e.chunkText("doc#0")).contains("w0 w1 w2 w3");
        assertThat(e.chunkText("doc#1")).contains("w4 w5 w6 w7");
        assertThat(e.chunkText("doc#2")).contains("w8 w9");
        assertThat(e.chunkText("doc#3")).isEmpty();
        assertThat(e.chunkCount()).isEqualTo(3);
    }

    @Test
    void shouldRepeatTheOverlapWordsAtTheStartOfTheNextChunk() {
        RagEngine e = engine();
        e.setChunking(4, 1);

        assertThat(e.ingest("doc", Fakes.words("w", 10))).isEqualTo(3);

        assertThat(e.chunkText("doc#0")).contains("w0 w1 w2 w3");
        assertThat(e.chunkText("doc#1")).contains("w3 w4 w5 w6");
        assertThat(e.chunkText("doc#2")).contains("w6 w7 w8 w9");
    }

    @Test
    void shouldNotCreateATrailingChunkThatIsAlreadyCovered() {
        RagEngine e = engine();
        e.setChunking(4, 1);

        assertThat(e.ingest("seven", Fakes.words("w", 7))).isEqualTo(2);
        assertThat(e.ingest("four", Fakes.words("w", 4))).isEqualTo(1);
        assertThat(e.ingest("one", "w0")).isEqualTo(1);
        assertThat(e.ingest("five", Fakes.words("w", 5))).isEqualTo(2);
        assertThat(e.chunkText("five#1")).contains("w3 w4");
    }

    @Test
    void shouldCreateNoChunksForEmptyOrBlankText() {
        RagEngine e = engine();

        assertThat(e.ingest("doc", "")).isZero();
        assertThat(e.ingest("doc", "  \n\t ")).isZero();
        assertThat(e.chunkCount()).isZero();
        assertThat(e.retrieve("alpha", 5)).isEmpty();
    }

    @Test
    void shouldJoinChunkWordsWithSingleSpaces() {
        RagEngine e = engine();
        e.setChunking(3, 0);

        e.ingest("doc", "  alpha\n beta\t\tgamma   delta  ");

        assertThat(e.chunkText("doc#0")).contains("alpha beta gamma");
        assertThat(e.chunkText("doc#1")).contains("delta");
    }

    @Test
    void shouldReplaceAllChunksWhenADocumentIsIngestedAgain() {
        RagEngine e = engine();
        e.setChunking(2, 0);
        e.ingest("doc", "alpha beta gamma delta epsilon zeta");
        e.ingest("other", "eta theta");
        assertThat(e.chunkCount()).isEqualTo(4);

        assertThat(e.ingest("doc", "iota kappa")).isEqualTo(1);

        assertThat(e.chunkCount()).isEqualTo(2);
        assertThat(e.chunkText("doc#0")).contains("iota kappa");
        assertThat(e.chunkText("doc#1")).isEmpty();
        assertThat(e.chunkText("doc#2")).isEmpty();
        assertThat(e.retrieve("alpha", 10)).containsExactlyInAnyOrder("doc#0", "other#0");
        assertThat(e.ingest("doc", "")).isZero();
        assertThat(e.retrieve("alpha", 10)).containsExactly("other#0");
    }

    @Test
    void shouldUseTheConfiguredChunkingOnlyForLaterIngests() {
        RagEngine e = engine();
        e.setChunking(2, 0);
        e.ingest("old", "alpha beta gamma delta");

        e.setChunking(4, 0);
        e.ingest("new", "alpha beta gamma delta");

        assertThat(e.chunkText("old#1")).contains("gamma delta");
        assertThat(e.chunkText("new#0")).contains("alpha beta gamma delta");
        assertThat(e.chunkCount()).isEqualTo(3);
    }

    @Test
    void shouldDefaultToChunksOfAHundredWordsWithAnOverlapOfTwenty() {
        RagEngine e = engine();

        assertThat(e.ingest("doc", Fakes.words("w", 250))).isEqualTo(3);
        assertThat(e.chunkText("doc#1").orElseThrow()).startsWith("w80 w81").endsWith("w178 w179");
        assertThat(e.chunkText("doc#2").orElseThrow()).startsWith("w160 w161").endsWith("w248 w249");
    }

    @Test
    void shouldRejectInvalidChunkingAndKeepTheOldSetting() {
        RagEngine e = engine();
        e.setChunking(3, 0);

        assertThatThrownBy(() -> e.setChunking(0, 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.setChunking(-2, 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.setChunking(4, 4)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.setChunking(4, 7)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.setChunking(4, -1)).isInstanceOf(IllegalArgumentException.class);

        assertThat(e.ingest("doc", "w0 w1 w2 w3 w4 w5")).isEqualTo(2);
        e.setChunking(1, 0);
        assertThat(e.ingest("single", "w0 w1 w2")).isEqualTo(3);
    }

    @Test
    void shouldRejectBlankDocumentIdsAndNullText() {
        RagEngine e = engine();

        assertThatThrownBy(() -> e.ingest(null, "alpha")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.ingest("  ", "alpha")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.ingest("doc", null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.deleteDocument(" ")).isInstanceOf(IllegalArgumentException.class);
        assertThat(e.chunkCount()).isZero();
    }

    @Test
    void shouldRejectMissingCollaborators() {
        engine();

        assertThatThrownBy(() -> Solution.create(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(embedder, null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> Solution.create(null, new BruteForceStore()))
                .isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRankChunksByCosineSimilarity() {
        RagEngine e = engine();
        e.ingest("d1", "alpha alpha beta");
        e.ingest("d2", "alpha gamma gamma");
        e.ingest("d3", "delta delta delta");

        assertThat(e.retrieve("alpha", 3)).containsExactly("d1#0", "d2#0", "d3#0");
        assertThat(e.retrieve("gamma beta", 3)).containsExactly("d2#0", "d1#0", "d3#0");
        assertThat(e.retrieve("delta", 3)).first().isEqualTo("d3#0");
    }

    @Test
    void shouldIgnoreTheLengthOfAChunkWhenRanking() {
        RagEngine e = engine();
        e.ingest("short", "alpha");
        e.ingest("long", "alpha alpha alpha alpha");
        e.ingest("mixed", "alpha beta");

        assertThat(e.retrieve("alpha alpha", 3)).containsExactly("long#0", "short#0", "mixed#0");
    }

    @Test
    void shouldBreakTiesByChunkIdInAscendingStringOrder() {
        RagEngine e = engine();
        e.ingest("b", "alpha");
        e.ingest("a", "alpha");
        e.ingest("c", "alpha");

        assertThat(e.retrieve("alpha", 3)).containsExactly("a#0", "b#0", "c#0");
    }

    @Test
    void shouldOrderEqualScoresOfNumberedChunksAsStrings() {
        RagEngine e = engine();
        e.setChunking(1, 0);
        e.ingest("n", "alpha alpha alpha alpha alpha alpha alpha alpha alpha alpha alpha alpha");

        assertThat(e.retrieve("alpha", 12))
                .containsExactly("n#0", "n#1", "n#10", "n#11", "n#2", "n#3", "n#4", "n#5", "n#6", "n#7", "n#8", "n#9");
    }

    @Test
    void shouldReturnAtMostKResultsAndAllChunksWhenThereAreFewer() {
        RagEngine e = engine();
        e.ingest("d1", "alpha");
        e.ingest("d2", "beta");
        e.ingest("d3", "gamma");

        assertThat(e.retrieve("alpha", 1)).containsExactly("d1#0");
        assertThat(e.retrieve("alpha", 2)).hasSize(2);
        assertThat(e.retrieve("alpha", 3)).hasSize(3);
        assertThat(e.retrieve("alpha", 1_000_000)).hasSize(3);
    }

    @Test
    void shouldStillReturnChunksWithZeroSimilarityAfterTheRelatedOnes() {
        RagEngine e = engine();
        e.ingest("x", "gamma");
        e.ingest("y", "alpha");
        e.ingest("z", "unknownword");

        assertThat(e.retrieve("alpha", 3)).containsExactly("y#0", "x#0", "z#0");
    }

    @Test
    void shouldReturnNothingWhenNothingIsIngested() {
        RagEngine e = engine();

        assertThat(e.retrieve("alpha", 3)).isEmpty();
        assertThat(e.buildPrompt("alpha", 3, 1_000)).isEqualTo(prompt("(none)", "alpha"));
    }

    @Test
    void shouldRejectBadRetrieveArguments() {
        RagEngine e = engine();
        e.ingest("d", "alpha");

        assertThatThrownBy(() -> e.retrieve("alpha", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.retrieve("alpha", -1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.retrieve(null, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.retrieve("  ", 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.buildPrompt("alpha", 0, 100)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.buildPrompt(" ", 1, 100)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.buildPrompt("alpha", 1, -1)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldBuildThePromptFromTheExactTemplate() {
        RagEngine e = engine();
        ingestPromptCorpus(e);

        assertThat(e.buildPrompt("alpha", 3, 1_000))
                .isEqualTo("Answer using only the context.\n\nContext:\n"
                        + "[1] alpha\n[2] alpha beta\n[3] alpha gamma delta\n\nQuestion: alpha\nAnswer:");
        assertThat(e.buildPrompt("alpha", 1, 1_000)).isEqualTo(prompt("[1] alpha", "alpha"));
        assertThat(e.buildPrompt("alpha", 1_000, 1_000)).contains("[3] alpha gamma delta");
    }

    @Test
    void shouldIncludeWholeChunksOnlyWhileTheContextFitsTheBudget() {
        RagEngine e = engine();
        ingestPromptCorpus(e);
        // context lengths: "[1] alpha" = 9, + "\n[2] alpha beta" = 24, + "\n[3] alpha gamma delta" = 46

        assertThat(e.buildPrompt("alpha", 3, 46)).contains("[3] alpha gamma delta");
        assertThat(e.buildPrompt("alpha", 3, 45)).isEqualTo(prompt("[1] alpha\n[2] alpha beta", "alpha"));
        assertThat(e.buildPrompt("alpha", 3, 24)).isEqualTo(prompt("[1] alpha\n[2] alpha beta", "alpha"));
        assertThat(e.buildPrompt("alpha", 3, 23)).isEqualTo(prompt("[1] alpha", "alpha"));
        assertThat(e.buildPrompt("alpha", 3, 9)).isEqualTo(prompt("[1] alpha", "alpha"));
    }

    @Test
    void shouldRenderNoContextWhenEvenTheFirstChunkDoesNotFit() {
        RagEngine e = engine();
        ingestPromptCorpus(e);

        assertThat(e.buildPrompt("alpha", 3, 8)).isEqualTo(prompt("(none)", "alpha"));
        assertThat(e.buildPrompt("alpha", 3, 0)).isEqualTo(prompt("(none)", "alpha"));
    }

    @Test
    void shouldStopAtTheFirstChunkThatDoesNotFitEvenIfALaterOneWould() {
        RagEngine e = engine();
        e.ingest("c", "alpha");
        e.ingest("big", "alpha alpha alpha beta");
        e.ingest("small", "alpha beta beta");
        // ranking for "alpha": c (1.0), big, small; entries are 9, 26 and 19 characters long

        assertThat(e.retrieve("alpha", 3)).containsExactly("c#0", "big#0", "small#0");
        assertThat(e.buildPrompt("alpha", 3, 30)).isEqualTo(prompt("[1] alpha", "alpha"));
    }

    @Test
    void shouldCountTheWidthOfMultiDigitRanksInTheBudget() {
        RagEngine e = engine();
        e.setChunking(1, 0);
        e.ingest("n", "alpha alpha alpha alpha alpha alpha alpha alpha alpha alpha alpha");
        // 11 chunks "alpha": entries "[1] alpha".."[9] alpha" are 9 chars, "[10] alpha" and "[11] alpha" are 10
        // 9 entries = 9*9 + 8 newlines = 89; the 10th adds 1 + 10 = 11 -> 100; the 11th -> 111

        assertThat(e.buildPrompt("alpha", 11, 100)).contains("[10] alpha").doesNotContain("[11]");
        assertThat(e.buildPrompt("alpha", 11, 99)).contains("[9] alpha").doesNotContain("[10]");
        assertThat(e.buildPrompt("alpha", 11, 111)).contains("[11] alpha");
    }

    @Test
    void shouldPutTheQueryVerbatimIntoThePrompt() {
        RagEngine e = engine();
        e.ingest("d", "alpha");

        assertThat(e.buildPrompt("  what is alpha?\nexplain ", 1, 100))
                .isEqualTo(prompt("[1] alpha", "  what is alpha?\nexplain "));
    }

    @Test
    void shouldDeleteOnlyTheChunksOfThatDocument() {
        RagEngine e = engine();
        e.setChunking(2, 0);
        e.ingest("a", "alpha beta gamma delta");
        e.ingest("b", "alpha beta");

        assertThat(e.deleteDocument("a")).isTrue();

        assertThat(e.chunkCount()).isEqualTo(1);
        assertThat(e.chunkText("a#0")).isEmpty();
        assertThat(e.retrieve("alpha", 10)).containsExactly("b#0");
        assertThat(e.deleteDocument("a")).isFalse();
        assertThat(e.deleteDocument("never-ingested")).isFalse();
    }

    @Test
    void shouldAllowAnIngestAfterADelete() {
        RagEngine e = engine();
        e.ingest("a", "alpha");
        e.deleteDocument("a");

        assertThat(e.ingest("a", "beta")).isEqualTo(1);

        assertThat(e.chunkText("a#0")).contains("beta");
        assertThat(e.retrieve("beta", 5)).containsExactly("a#0");
    }

    @Test
    void shouldWorkWithAnotherEmbedder() {
        RagEngine e = Candidate.of(() -> Solution.create(new LetterEmbedder()));
        e.ingest("a", "aaa");
        e.ingest("b", "bbb");
        e.ingest("c", "abab");

        assertThat(e.retrieve("bbb", 3)).containsExactly("b#0", "c#0", "a#0");
        assertThat(e.retrieve("aaa", 3)).containsExactly("a#0", "c#0", "b#0");
    }

    @Test
    void shouldCallTheEmbedderOncePerChunkAndOncePerQuery() {
        RagEngine e = engine();
        e.setChunking(2, 0);

        e.ingest("doc", "alpha beta gamma delta epsilon");
        assertThat(embedder.calls.get()).isEqualTo(3);

        e.retrieve("alpha", 2);
        assertThat(embedder.calls.get()).isEqualTo(4);
        e.buildPrompt("alpha", 2, 100);
        assertThat(embedder.calls.get()).isEqualTo(5);

        e.chunkCount();
        e.chunkText("doc#0");
        e.deleteDocument("doc");
        assertThat(embedder.calls.get()).isEqualTo(5);
    }

    @Test
    void shouldStoreVectorsInTheSuppliedVectorStore() {
        BruteForceStore store = new BruteForceStore();
        RagEngine e = Candidate.of(() -> Solution.create(embedder, store));
        e.setChunking(2, 0);

        e.ingest("a", "alpha beta gamma delta");
        assertThat(store.ids()).containsExactly("a#0", "a#1");
        assertThat(e.retrieve("gamma", 2)).containsExactly("a#1", "a#0");
        assertThat(store.searches.get()).isPositive();

        e.ingest("a", "alpha beta");
        assertThat(store.ids()).containsExactly("a#0");

        e.ingest("b", "gamma");
        e.deleteDocument("a");
        assertThat(store.ids()).containsExactly("b#0");
        e.deleteDocument("b");
        assertThat(store.ids()).isEmpty();
    }

    @Test
    void shouldGiveTheSameAnswersWithTheDefaultStoreAndASuppliedStore() {
        RagEngine plain = engine();
        RagEngine custom = Candidate.of(() -> Solution.create(new VocabEmbedder(), new BruteForceStore()));
        String[][] docs = {
            {"d1", "alpha beta gamma alpha"}, {"d2", "delta epsilon alpha"}, {"d3", "zeta eta theta iota"},
            {"d4", "alpha"}, {"d5", "gamma gamma gamma beta"}, {"d6", "nothing known here"}
        };
        for (RagEngine e : List.of(plain, custom)) {
            e.setChunking(3, 1);
            for (String[] d : docs) {
                e.ingest(d[0], d[1]);
            }
        }

        for (String q : List.of("alpha", "gamma beta", "zeta iota", "mu nu", "alpha delta epsilon")) {
            assertThat(custom.retrieve(q, 4)).isEqualTo(plain.retrieve(q, 4));
            assertThat(custom.buildPrompt(q, 4, 80)).isEqualTo(plain.buildPrompt(q, 4, 80));
        }
    }

    @Test
    void shouldPropagateEmbedderFailuresAndKeepThePreviousVersionOfTheDocument() {
        RagEngine e = Candidate.of(() -> Solution.create(text -> {
            if (text.contains("boom")) {
                throw new IllegalStateException("embedding service down");
            }
            return embedder.embed(text);
        }));
        e.setChunking(2, 0);
        e.ingest("doc", "alpha beta gamma delta");

        assertThatThrownBy(() -> e.ingest("doc", "iota kappa lambda boom"))
                .isInstanceOf(IllegalStateException.class);

        assertThat(e.chunkCount()).isEqualTo(2);
        assertThat(e.chunkText("doc#0")).contains("alpha beta");
        assertThat(e.chunkText("doc#1")).contains("gamma delta");
    }

    @Test
    void shouldHandleConcurrentIngestsAndRetrievals() {
        RagEngine e = engine();
        e.setChunking(2, 0);

        Concurrently.run(8, t -> {
            for (int d = 0; d < 20; d++) {
                e.ingest("t" + t + "-d" + d, "alpha beta gamma delta epsilon zeta");
                List<String> hits = e.retrieve("alpha", 5);
                assertThat(hits).hasSizeBetween(1, 5);
                e.buildPrompt("beta", 3, 200);
            }
        });

        assertThat(e.chunkCount()).isEqualTo(8 * 20 * 3);
        assertThat(e.retrieve("alpha", 1_000)).hasSize(480).doesNotHaveDuplicates();
    }

    @Test
    void shouldNotLoseDocumentsWhenSomeAreDeletedConcurrently() {
        RagEngine e = engine();
        e.setChunking(2, 0);

        Concurrently.run(8, t -> {
            for (int d = 0; d < 25; d++) {
                String id = "t" + t + "-" + d;
                e.ingest(id, "alpha beta gamma");
                if (d % 2 == 0) {
                    e.deleteDocument(id);
                }
            }
        });

        List<String> all = new ArrayList<>(e.retrieve("alpha", 10_000));
        assertThat(all).hasSize(8 * 12 * 2);
        assertThat(new TreeSet<>(all)).hasSize(all.size());
        assertThat(e.chunkCount()).isEqualTo(all.size());
    }
}

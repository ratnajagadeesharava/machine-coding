package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Embedder;
import contract.RagEngine;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.concurrent.CountDownLatch;
import java.util.concurrent.ExecutorService;
import java.util.concurrent.Executors;
import java.util.concurrent.Future;
import java.util.concurrent.TimeUnit;
import java.util.concurrent.atomic.AtomicBoolean;
import org.junit.jupiter.api.Test;
import playground.Fakes.BruteForceStore;
import playground.Fakes.VocabEmbedder;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    @Test
    void shouldKeepServingOtherDocumentsWhileOneEmbeddingIsSlow() throws Exception {
        VocabEmbedder inner = new VocabEmbedder();
        CountDownLatch entered = new CountDownLatch(1);
        CountDownLatch release = new CountDownLatch(1);
        Embedder gated = text -> {
            if (text.contains("SLOW")) {
                entered.countDown();
                try {
                    if (!release.await(20, TimeUnit.SECONDS)) {
                        throw new IllegalStateException("never released");
                    }
                } catch (InterruptedException e) {
                    Thread.currentThread().interrupt();
                    throw new IllegalStateException(e);
                }
            }
            return inner.embed(text);
        };
        RagEngine e = Candidate.of(() -> Solution.create(gated));
        e.ingest("fast0", "alpha beta");
        ExecutorService pool = Executors.newFixedThreadPool(2);
        try {
            Future<?> slow = pool.submit(() -> e.ingest("slow", "SLOW alpha gamma"));
            assertThat(entered.await(10, TimeUnit.SECONDS)).isTrue();

            Future<List<String>> others = pool.submit(() -> {
                e.ingest("fast1", "alpha");
                e.deleteDocument("fast0");
                e.buildPrompt("alpha", 3, 100);
                return e.retrieve("alpha", 5);
            });
            List<String> seen;
            try {
                seen = others.get(5, TimeUnit.SECONDS);
            } catch (java.util.concurrent.TimeoutException t) {
                throw new AssertionError("other operations were blocked by a slow embedding of one document");
            } finally {
                release.countDown();
            }
            assertThat(seen).containsExactly("fast1#0");
            slow.get(10, TimeUnit.SECONDS);
            assertThat(e.retrieve("alpha", 5)).containsExactly("fast1#0", "slow#0");
        } finally {
            release.countDown();
            pool.shutdownNow();
        }
    }

    @Test
    void shouldNeverShowAHalfReplacedDocument() {
        RagEngine e = Candidate.of(() -> Solution.create(new VocabEmbedder()));
        e.setChunking(1, 0);
        String versionA = "alpha alpha alpha alpha";
        String versionB = "beta beta";
        e.ingest("d", versionA);
        AtomicBoolean done = new AtomicBoolean();

        Concurrently.run(6, 60_000, t -> {
            if (t < 2) {
                for (int i = 0; i < 400; i++) {
                    e.ingest("d", (i + t) % 2 == 0 ? versionA : versionB);
                }
                if (t == 0) {
                    done.set(true);
                }
                return;
            }
            while (!done.get()) {
                List<String> ids = e.retrieve("mu", 100);
                assertThat(ids.size()).isIn(2, 4);
                String p = e.buildPrompt("mu", 100, 10_000);
                boolean a = p.contains("alpha");
                boolean b = p.contains("beta");
                assertThat(a ^ b).as("prompt mixes versions: " + p).isTrue();
            }
        });
        assertThat(e.chunkCount()).isIn(2, 4);
    }

    @Test
    void shouldNotConfuseDocumentsWhoseIdsShareAPrefix() {
        RagEngine e = Candidate.of(() -> Solution.create(new VocabEmbedder()));
        e.setChunking(1, 0);
        e.ingest("a", "alpha beta");
        e.ingest("ab", "alpha");
        e.ingest("a#1", "alpha");

        assertThat(e.retrieve("alpha", 10)).containsExactly("a#0", "a#1#0", "ab#0", "a#1");

        assertThat(e.deleteDocument("a")).isTrue();

        assertThat(e.retrieve("alpha", 10)).containsExactly("a#1#0", "ab#0");
        assertThat(e.chunkText("a#1#0")).contains("alpha");
        assertThat(e.deleteDocument("a#1")).isTrue();
        assertThat(e.retrieve("alpha", 10)).containsExactly("ab#0");
    }

    @Test
    void shouldIngestFromManyThreadsWithoutLosingAnything() {
        RagEngine e = Candidate.of(() -> Solution.create(new VocabEmbedder()));
        e.setChunking(3, 0);

        Concurrently.run(16, 60_000, t -> {
            for (int d = 0; d < 50; d++) {
                e.ingest("t" + t + "-" + d, "alpha beta gamma delta epsilon zeta eta");
            }
        });

        assertThat(e.chunkCount()).isEqualTo(16 * 50 * 3);
        List<String> all = e.retrieve("alpha", 100_000);
        assertThat(all).hasSize(2_400).doesNotHaveDuplicates();
        assertThat(e.chunkText("t15-49#2")).contains("eta");
    }

    @Test
    void shouldRankALargeCorpusLikeABruteForceScan() {
        VocabEmbedder reference = new VocabEmbedder();
        RagEngine e = Candidate.of(() -> Solution.create(new VocabEmbedder()));
        Map<String, double[]> vectors = new HashMap<>();
        long seed = 12345;
        for (int d = 0; d < 4_000; d++) {
            StringBuilder sb = new StringBuilder();
            int n = 2 + (int) ((seed >>> 33) % 7);
            for (int i = 0; i < n; i++) {
                seed = (seed * 6364136223846793005L + 1442695040888963407L);
                sb.append(Fakes.WORDS[(int) ((seed >>> 33) % Fakes.WORDS.length)]).append(' ');
            }
            e.ingest("doc" + d, sb.toString());
            vectors.put("doc" + d + "#0", reference.embed(sb.toString()));
        }

        for (int q = 0; q < 60; q++) {
            seed = (seed * 6364136223846793005L + 1442695040888963407L);
            String query = Fakes.WORDS[(int) ((seed >>> 33) % 16)] + " " + Fakes.WORDS[(int) ((seed >>> 40) % 16)];
            double[] qv = reference.embed(query);
            List<Double> expected = new ArrayList<>();
            vectors.values().forEach(v -> expected.add(Fakes.cosine(qv, v)));
            expected.sort(java.util.Comparator.reverseOrder());

            List<String> got = e.retrieve(query, 10);

            assertThat(got).hasSize(10).doesNotHaveDuplicates();
            for (int i = 0; i < 10; i++) {
                assertThat(Fakes.cosine(qv, vectors.get(got.get(i)))).isCloseTo(expected.get(i), org.assertj.core.data.Offset.offset(1e-9));
            }
        }
    }

    @Test
    void shouldLeaveNoTraceWhenTheFirstIngestOfADocumentFailsHalfway() {
        VocabEmbedder inner = new VocabEmbedder();
        BruteForceStore store = new BruteForceStore();
        RagEngine e = Candidate.of(() -> Solution.create(text -> {
            if (text.contains("zeta")) {
                throw new IllegalStateException("embedding service down");
            }
            return inner.embed(text);
        }, store));
        e.setChunking(2, 0);

        assertThatThrownBy(() -> e.ingest("doc", "alpha beta gamma delta zeta eta"))
                .isInstanceOf(IllegalStateException.class);

        assertThat(e.chunkCount()).isZero();
        assertThat(store.ids()).isEmpty();
        assertThat(e.retrieve("alpha", 5)).isEmpty();
        assertThat(e.ingest("doc", "alpha beta")).isEqualTo(1);
        assertThatThrownBy(() -> e.retrieve("zeta", 1)).isInstanceOf(IllegalStateException.class);
        assertThat(e.chunkCount()).isEqualTo(1);
    }

    @Test
    void shouldOrderEverythingByIdWhenTheQueryMatchesNothing() {
        RagEngine e = Candidate.of(() -> Solution.create(new VocabEmbedder()));
        e.ingest("m", "alpha");
        e.ingest("k", "beta");
        e.ingest("z", "unknown words only");

        assertThat(e.retrieve("totally unrelated", 5)).containsExactly("k#0", "m#0", "z#0");
        assertThat(e.buildPrompt("totally unrelated", 2, 100))
                .isEqualTo("Answer using only the context.\n\nContext:\n[1] beta\n[2] alpha\n\nQuestion: totally unrelated\nAnswer:");
    }

    @Test
    void shouldHandleAVeryLongDocument() {
        RagEngine e = Candidate.of(() -> Solution.create(new VocabEmbedder()));

        int chunks = e.ingest("big", Fakes.words("w", 100_000));

        assertThat(chunks).isEqualTo(1_250);
        assertThat(e.chunkCount()).isEqualTo(1_250);
        assertThat(e.chunkText("big#1249").orElseThrow()).startsWith("w99920 ").endsWith(" w99999");
    }

    @Test
    void shouldGiveTheSameStateWhenTheSameDocumentIsIngestedTwice() {
        VocabEmbedder embedder = new VocabEmbedder();
        BruteForceStore store = new BruteForceStore();
        RagEngine e = Candidate.of(() -> Solution.create(embedder, store));
        e.setChunking(2, 1);

        int first = e.ingest("doc", "alpha beta gamma delta epsilon");
        List<String> before = e.retrieve("gamma", 10);
        String promptBefore = e.buildPrompt("gamma", 10, 500);
        int second = e.ingest("doc", "alpha beta gamma delta epsilon");

        assertThat(second).isEqualTo(first).isEqualTo(4);
        assertThat(e.chunkCount()).isEqualTo(4);
        assertThat(store.ids()).containsExactly("doc#0", "doc#1", "doc#2", "doc#3");
        assertThat(e.retrieve("gamma", 10)).isEqualTo(before);
        assertThat(e.buildPrompt("gamma", 10, 500)).isEqualTo(promptBefore);
    }
}

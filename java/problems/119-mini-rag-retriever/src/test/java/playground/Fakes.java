package playground;

import contract.Embedder;
import contract.VectorStore;
import java.util.ArrayList;
import java.util.Comparator;
import java.util.HashMap;
import java.util.List;
import java.util.Locale;
import java.util.Map;
import java.util.TreeSet;
import java.util.concurrent.atomic.AtomicInteger;

/** Deterministic collaborators implemented by the tests. */
final class Fakes {
    private Fakes() {}

    static final String[] WORDS = {
        "alpha", "beta", "gamma", "delta", "epsilon", "zeta", "eta", "theta",
        "iota", "kappa", "lambda", "mu", "nu", "xi", "omicron", "pi"
    };

    /** Counts occurrences of 16 known words (lower-cased, split on whitespace); every other word is ignored. */
    static class VocabEmbedder implements Embedder {
        final AtomicInteger calls = new AtomicInteger();
        private final Map<String, Integer> index = new HashMap<>();

        VocabEmbedder() {
            for (int i = 0; i < WORDS.length; i++) {
                index.put(WORDS[i], i);
            }
        }

        @Override
        public double[] embed(String text) {
            calls.incrementAndGet();
            double[] v = new double[WORDS.length];
            for (String w : text.toLowerCase(Locale.ROOT).split("\\s+")) {
                Integer i = index.get(w);
                if (i != null) {
                    v[i] += 1;
                }
            }
            return v;
        }
    }

    /** A completely different embedding: a 26-letter histogram. */
    static class LetterEmbedder implements Embedder {
        @Override
        public double[] embed(String text) {
            double[] v = new double[26];
            for (char c : text.toLowerCase(Locale.ROOT).toCharArray()) {
                if (c >= 'a' && c <= 'z') {
                    v[c - 'a'] += 1;
                }
            }
            return v;
        }
    }

    /** Plain scan over everything stored; records how it was used. */
    static class BruteForceStore implements VectorStore {
        private record Entry(double[] v, String payload) {}

        private final Map<String, Entry> entries = new HashMap<>();
        final AtomicInteger upserts = new AtomicInteger();
        final AtomicInteger removes = new AtomicInteger();
        final AtomicInteger searches = new AtomicInteger();

        @Override
        public synchronized void upsert(String id, double[] vector, String payload) {
            upserts.incrementAndGet();
            entries.put(id, new Entry(vector.clone(), payload));
        }

        @Override
        public synchronized boolean remove(String id) {
            removes.incrementAndGet();
            return entries.remove(id) != null;
        }

        @Override
        public synchronized List<Hit> topK(double[] q, int k) {
            searches.incrementAndGet();
            List<Hit> all = new ArrayList<>();
            for (Map.Entry<String, Entry> e : entries.entrySet()) {
                all.add(new Hit(e.getKey(), cosine(q, e.getValue().v()), e.getValue().payload()));
            }
            all.sort(Comparator.comparingDouble(Hit::score).reversed().thenComparing(Hit::id));
            return new ArrayList<>(all.subList(0, Math.min(k, all.size())));
        }

        synchronized TreeSet<String> ids() {
            return new TreeSet<>(entries.keySet());
        }
    }

    static double cosine(double[] a, double[] b) {
        double dot = 0;
        double na = 0;
        double nb = 0;
        for (int i = 0; i < a.length; i++) {
            dot += a[i] * b[i];
            na += a[i] * a[i];
            nb += b[i] * b[i];
        }
        return na == 0 || nb == 0 ? 0 : dot / (Math.sqrt(na) * Math.sqrt(nb));
    }

    static String words(String prefix, int n) {
        StringBuilder sb = new StringBuilder();
        for (int i = 0; i < n; i++) {
            sb.append(i == 0 ? "" : " ").append(prefix).append(i);
        }
        return sb.toString();
    }
}

package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.WordCounter;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.List;
import java.util.Map;
import java.util.Random;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private WordCounter counter() {
        return Candidate.of(Solution::create);
    }

    /** Straightforward single-threaded reference used to compute the expected answer. */
    private static Map<String, Integer> expected(List<String> docs) {
        Map<String, Integer> out = new HashMap<>();
        for (String d : docs) {
            StringBuilder w = new StringBuilder();
            for (int i = 0; i <= d.length(); i++) {
                char c = i < d.length() ? d.charAt(i) : ' ';
                if ((c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z')) {
                    w.append(Character.toLowerCase(c));
                } else if (w.length() > 0) {
                    out.merge(w.toString(), 1, Integer::sum);
                    w.setLength(0);
                }
            }
        }
        return out;
    }

    private static final String[] VOCAB = vocabulary(64);

    private static String[] vocabulary(int n) {
        String[] v = new String[n];
        Random r = new Random(11);
        for (int i = 0; i < n; i++) {
            StringBuilder sb = new StringBuilder();
            int len = 3 + r.nextInt(10);
            for (int k = 0; k < len; k++) {
                sb.append((char) ('a' + r.nextInt(26)));
            }
            v[i] = sb.toString() + (char) ('a' + i % 26); // keep words distinct enough
        }
        return v;
    }

    private static String document(Random r, int words) {
        StringBuilder sb = new StringBuilder();
        String[] seps = {" ", ", ", ".\n", " - ", "\t", "! ", "123 ", "_"};
        for (int i = 0; i < words; i++) {
            String w = VOCAB[r.nextInt(VOCAB.length)];
            sb.append(r.nextBoolean() ? w : w.toUpperCase()).append(seps[r.nextInt(seps.length)]);
        }
        return sb.toString();
    }

    private static List<String> documents(int docs, int wordsEach, long seed) {
        Random r = new Random(seed);
        List<String> out = new ArrayList<>();
        for (int i = 0; i < docs; i++) {
            out.add(document(r, wordsEach));
        }
        return out;
    }

    @Test
    void shouldCountWordsCaseInsensitively() {
        WordCounter c = counter();

        Map<String, Integer> result = c.count(List.of("Hello, hello WORLD!", "world of Hello"), 2);

        assertThat(result).containsOnly(Map.entry("hello", 3), Map.entry("world", 2), Map.entry("of", 1));
    }

    @Test
    void shouldTreatEverythingButAsciiLettersAsSeparators() {
        WordCounter c = counter();

        Map<String, Integer> result = c.count(List.of("abc123def a-b_c\nd\te", "x.y,z;é"), 1);

        assertThat(result)
                .containsOnly(
                        Map.entry("abc", 1), Map.entry("def", 1), Map.entry("a", 1), Map.entry("b", 1),
                        Map.entry("c", 1), Map.entry("d", 1), Map.entry("e", 1), Map.entry("x", 1),
                        Map.entry("y", 1), Map.entry("z", 1));
    }

    @Test
    void shouldHandleEmptyInput() {
        WordCounter c = counter();

        assertThat(c.count(List.of(), 4)).isEmpty();
        assertThat(c.count(List.of("", "   ", "123 !!"), 4)).isEmpty();
    }

    @Test
    void shouldRejectNonPositiveParallelism() {
        WordCounter c = counter();

        assertThatThrownBy(() -> c.count(List.of("a"), 0)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldMatchTheSequentialAnswerForEveryParallelism() {
        WordCounter c = counter();
        List<String> docs = documents(200, 500, 1);
        Map<String, Integer> expected = expected(docs);

        for (int p : new int[] {1, 2, 3, 4, 8, 16, 64}) {
            assertThat(c.count(docs, p)).as("parallelism %d", p).isEqualTo(expected);
        }
    }

    @Test
    void shouldCountOneHugeDocumentCorrectly() {
        WordCounter c = counter();
        // one document: a split inside it must never cut a word in half
        List<String> docs = List.of(document(new Random(5), 300_000));
        Map<String, Integer> expected = expected(docs);

        for (int p : new int[] {1, 4, 8}) {
            assertThat(c.count(docs, p)).as("parallelism %d", p).isEqualTo(expected);
        }
    }

    @Test
    void shouldHandleFewerDocumentsThanThreads() {
        WordCounter c = counter();
        List<String> docs = documents(3, 1_000, 9);

        assertThat(c.count(docs, 32)).isEqualTo(expected(docs));
    }

    @Test
    void shouldNotModifyTheInputAndAcceptImmutableLists() {
        WordCounter c = counter();
        List<String> docs = List.copyOf(documents(50, 100, 3));

        Map<String, Integer> result = c.count(docs, 4);

        assertThat(docs).isEqualTo(List.copyOf(docs));
        assertThat(result).isEqualTo(expected(docs));
    }

    @Test
    void shouldAddUpToTheTotalNumberOfWords() {
        WordCounter c = counter();
        List<String> docs = documents(100, 300, 21);

        int total = c.count(docs, 8).values().stream().mapToInt(Integer::intValue).sum();

        assertThat(total).isEqualTo(100 * 300);
    }

    @Test
    void shouldBeUsableFromSeveralThreadsAtOnce() {
        WordCounter c = counter();

        Concurrently.run(6, t -> {
            List<String> docs = documents(60, 200, 100 + t);
            assertThat(c.count(docs, 4)).isEqualTo(expected(docs));
        });
    }

    @Test
    void shouldLoseNoCountsOver100Runs() {
        WordCounter c = counter();
        Random r = new Random(77);
        for (int run = 0; run < 100; run++) {
            List<String> docs = documents(1 + r.nextInt(80), 1 + r.nextInt(300), run);
            int p = 1 + r.nextInt(12);

            assertThat(c.count(docs, p)).as("run %d, parallelism %d", run, p).isEqualTo(expected(docs));
        }
    }
}

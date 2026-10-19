package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.WordCounter;
import java.util.ArrayList;
import java.util.List;
import java.util.Map;
import java.util.Random;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class HiddenScenariosTest {

    @Test
    void shouldCountOneMillionWordsOfAVeryHotVocabulary() {
        WordCounter c = Candidate.of(Solution::create);
        List<String> docs = new ArrayList<>();
        for (int d = 0; d < 1_000; d++) {
            docs.add("the quick brown fox jumps over the lazy dog ".repeat(125)); // 1_000 words each
        }

        Map<String, Integer> result = c.count(docs, 16);

        assertThat(result.get("the")).isEqualTo(2 * 125 * 1_000);
        assertThat(result.get("dog")).isEqualTo(125 * 1_000);
        assertThat(result.values().stream().mapToInt(Integer::intValue).sum()).isEqualTo(9 * 125 * 1_000);
    }

    @Test
    void shouldCountMillionsOfDistinctWords() {
        WordCounter c = Candidate.of(Solution::create);
        Random r = new Random(4);
        List<String> docs = new ArrayList<>();
        for (int d = 0; d < 100; d++) {
            StringBuilder sb = new StringBuilder();
            for (int w = 0; w < 5_000; w++) {
                for (int k = 0; k < 6; k++) {
                    sb.append((char) ('a' + r.nextInt(26)));
                }
                sb.append(' ');
            }
            docs.add(sb.toString());
        }

        Map<String, Integer> result = c.count(docs, 8);

        assertThat(result.values().stream().mapToInt(Integer::intValue).sum()).isEqualTo(100 * 5_000);
    }
}

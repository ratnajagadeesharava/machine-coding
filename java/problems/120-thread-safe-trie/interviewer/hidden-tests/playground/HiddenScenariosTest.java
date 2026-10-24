package playground;

import static org.assertj.core.api.Assertions.assertThat;

import contract.WordSet;
import java.util.ArrayList;
import java.util.Collections;
import java.util.List;
import java.util.Random;
import java.util.TreeSet;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class HiddenScenariosTest {

    private static String word(int i) {
        return Integer.toString(i, 36);
    }

    @Test
    void shouldMatchABruteForceModelOnRandomSingleThreadedOperations() {
        WordSet t = Candidate.of(Solution::create);
        TreeSet<String> model = new TreeSet<>();
        Random rnd = new Random(42);
        String alphabet = "ab-1";

        for (int step = 0; step < 20_000; step++) {
            StringBuilder sb = new StringBuilder();
            int len = 1 + rnd.nextInt(5);
            for (int i = 0; i < len; i++) {
                sb.append(alphabet.charAt(rnd.nextInt(alphabet.length())));
            }
            String w = rnd.nextBoolean() ? sb.toString() : sb.toString().toUpperCase();
            String norm = w.toLowerCase();
            switch (rnd.nextInt(4)) {
                case 0 -> assertThat(t.insert(w)).isEqualTo(model.add(norm));
                case 1 -> assertThat(t.delete(w)).isEqualTo(model.remove(norm));
                case 2 -> assertThat(t.contains(w)).isEqualTo(model.contains(norm));
                default -> {
                    String p = norm.substring(0, rnd.nextInt(norm.length() + 1));
                    List<String> expected = new ArrayList<>();
                    for (String m : model.tailSet(p)) {
                        if (!m.startsWith(p)) {
                            break;
                        }
                        expected.add(m);
                    }
                    int limit = 1 + rnd.nextInt(8);
                    assertThat(t.startsWith(p)).isEqualTo(!expected.isEmpty());
                    assertThat(t.countWithPrefix(p)).isEqualTo(expected.size());
                    assertThat(t.wordsWithPrefix(p, limit))
                            .isEqualTo(expected.subList(0, Math.min(limit, expected.size())));
                }
            }
            assertThat(t.size()).isEqualTo(model.size());
        }
    }

    @Test
    void shouldReleaseEverythingWhenAllWordsAreDeletedInAnyOrder() {
        WordSet t = Candidate.of(Solution::create);
        List<String> words = new ArrayList<>();
        for (int i = 0; i < 3_000; i++) {
            words.add(word(i));
            t.insert(word(i));
        }
        Collections.shuffle(words, new Random(7));
        for (String w : words) {
            assertThat(t.delete(w)).isTrue();
            assertThat(t.contains(w)).isFalse();
        }

        assertThat(t.size()).isZero();
        assertThat(t.startsWith("")).isFalse();
        assertThat(t.countWithPrefix("1")).isZero();
        assertThat(t.wordsWithPrefix("", 10)).isEmpty();
        assertThat(t.insert("again")).isTrue();
        assertThat(t.wordsWithPrefix("", 10)).containsExactly("again");
    }

    @Test
    void shouldHandleTwoHundredThousandWordsWithManyThreadsInBoundedTime() {
        WordSet t = Candidate.of(Solution::create);

        Concurrently.run(8, 60_000, id -> {
            for (int i = id; i < 200_000; i += 8) {
                t.insert(word(i));
            }
        });

        assertThat(t.size()).isEqualTo(200_000);
        for (int i = 0; i < 20_000; i++) {
            assertThat(t.wordsWithPrefix(word(i * 5), 3)).isNotEmpty();
        }
    }

    @Test
    void shouldKeepCountWithPrefixConsistentWithTheListingWhileWritersRun() {
        WordSet t = Candidate.of(Solution::create);
        AtomicBoolean done = new AtomicBoolean();
        AtomicInteger writersLeft = new AtomicInteger(2);

        Concurrently.run(6, 60_000, id -> {
            if (id < 2) {
                for (int round = 0; round < 30; round++) {
                    for (int i = id; i < 1_000; i += 2) {
                        t.insert("k" + word(i));
                    }
                    for (int i = id; i < 1_000; i += 2) {
                        t.delete("k" + word(i));
                    }
                }
                if (writersLeft.decrementAndGet() == 0) {
                    done.set(true);
                }
                return;
            }
            while (!done.get()) {
                int c = t.countWithPrefix("k");
                assertThat(c).isBetween(0, 1_000);
                assertThat(t.wordsWithPrefix("k", 5_000)).hasSizeLessThanOrEqualTo(1_000);
                assertThat(t.size()).isBetween(0, 1_000);
            }
        });

        assertThat(t.size()).isZero();
    }

    @Test
    void shouldNotLetADeleteOfAWordRemoveItsSiblingsInsertedAtTheSameTime() {
        WordSet t = Candidate.of(Solution::create);

        // "car" is repeatedly deleted while "card" / "care" (sharing its path) are inserted and kept
        Concurrently.run(8, 60_000, id -> {
            for (int i = 0; i < 5_000; i++) {
                switch (id % 4) {
                    case 0 -> {
                        t.insert("car");
                        t.delete("car");
                    }
                    case 1 -> t.insert("card" + (i % 50));
                    case 2 -> t.insert("care" + (i % 50));
                    default -> {
                        t.insert("ca");
                        t.delete("ca");
                    }
                }
            }
        });

        assertThat(t.countWithPrefix("card")).isEqualTo(50);
        assertThat(t.countWithPrefix("care")).isEqualTo(50);
        assertThat(t.size()).isEqualTo(100);
        assertThat(t.startsWith("car")).isTrue();
    }

    @Test
    void shouldStayCorrectUnderRepeatedRacesOnTheSameSmallWordSet() {
        WordSet t = Candidate.of(Solution::create);
        String[] words = {"a", "ab", "abc", "abd", "b", "ba"};

        List<Integer> net = Concurrently.collect(8, 60_000, id -> {
            int balance = 0;
            Random rnd = new Random(id);
            for (int i = 0; i < 30_000; i++) {
                String w = words[rnd.nextInt(words.length)];
                if (rnd.nextBoolean()) {
                    if (t.insert(w)) {
                        balance++;
                    }
                } else if (t.delete(w)) {
                    balance--;
                }
            }
            return balance;
        });

        int expected = net.stream().mapToInt(Integer::intValue).sum();
        assertThat(t.size()).isEqualTo(expected).isBetween(0, words.length);
        assertThat(t.wordsWithPrefix("", 10)).hasSize(expected).doesNotHaveDuplicates();
        int present = 0;
        for (String w : words) {
            if (t.contains(w)) {
                present++;
            }
        }
        assertThat(present).isEqualTo(expected);
    }

    @Test
    void shouldRejectInvalidInputWithoutDisturbingConcurrentWork() {
        WordSet t = Candidate.of(Solution::create);

        Concurrently.run(4, 30_000, id -> {
            for (int i = 0; i < 2_000; i++) {
                try {
                    t.insert(i % 2 == 0 ? "bad word" : "good" + id + "-" + i);
                } catch (IllegalArgumentException expectedForBadWord) {
                    // rejected calls must leave no trace
                }
            }
        });

        assertThat(t.size()).isEqualTo(4_000);
        assertThat(t.startsWith("bad")).isFalse();
    }
}

package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.WordSet;
import java.util.ArrayList;
import java.util.HashSet;
import java.util.List;
import java.util.Set;
import java.util.concurrent.atomic.AtomicBoolean;
import java.util.concurrent.atomic.AtomicInteger;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private WordSet trie() {
        return Candidate.of(Solution::create);
    }

    private static WordSet with(WordSet t, String... words) {
        for (String w : words) {
            t.insert(w);
        }
        return t;
    }

    /** A distinct word made only of allowed characters for every i. */
    private static String word(int i) {
        return Integer.toString(i, 36);
    }

    @Test
    void shouldReportWhetherAnInsertAddedTheWord() {
        WordSet t = trie();

        assertThat(t.insert("apple")).isTrue();
        assertThat(t.insert("apple")).isFalse();
        assertThat(t.size()).isEqualTo(1);
    }

    @Test
    void shouldFindOnlyExactWordsWithContains() {
        WordSet t = with(trie(), "apple", "app");

        assertThat(t.contains("apple")).isTrue();
        assertThat(t.contains("app")).isTrue();
        assertThat(t.contains("ap")).isFalse();
        assertThat(t.contains("apples")).isFalse();
        assertThat(t.contains("banana")).isFalse();
    }

    @Test
    void shouldAnswerStartsWithForPrefixesAndWholeWords() {
        WordSet t = with(trie(), "apple");

        assertThat(t.startsWith("a")).isTrue();
        assertThat(t.startsWith("appl")).isTrue();
        assertThat(t.startsWith("apple")).isTrue();
        assertThat(t.startsWith("apples")).isFalse();
        assertThat(t.startsWith("b")).isFalse();
    }

    @Test
    void shouldTreatTheEmptyPrefixAsEverything() {
        WordSet empty = trie();
        assertThat(empty.startsWith("")).isFalse();
        assertThat(empty.countWithPrefix("")).isZero();
        assertThat(empty.wordsWithPrefix("", 10)).isEmpty();

        WordSet t = with(trie(), "b", "a", "c");
        assertThat(t.startsWith("")).isTrue();
        assertThat(t.startsWith("   ")).isTrue();
        assertThat(t.countWithPrefix("")).isEqualTo(3);
        assertThat(t.wordsWithPrefix("", 10)).containsExactly("a", "b", "c");
    }

    @Test
    void shouldNormaliseWordsAndPrefixesByTrimmingAndLowerCasing() {
        WordSet t = trie();

        assertThat(t.insert("  Hello ")).isTrue();
        assertThat(t.insert("HELLO")).isFalse();
        assertThat(t.contains("hello")).isTrue();
        assertThat(t.contains(" HeLLo  ")).isTrue();
        assertThat(t.startsWith(" HEL ")).isTrue();
        assertThat(t.countWithPrefix("HE")).isEqualTo(1);
        assertThat(t.wordsWithPrefix(" H", 5)).containsExactly("hello");
        assertThat(t.delete(" HELLO ")).isTrue();
        assertThat(t.size()).isZero();
    }

    @Test
    void shouldAcceptDigitsAndHyphens() {
        WordSet t = with(trie(), "covid-19", "x-ray", "2024", "-");

        assertThat(t.contains("covid-19")).isTrue();
        assertThat(t.wordsWithPrefix("covid-", 5)).containsExactly("covid-19");
        assertThat(t.wordsWithPrefix("", 10)).containsExactly("-", "2024", "covid-19", "x-ray");
    }

    @Test
    void shouldRejectBlankNullAndForbiddenCharacters() {
        WordSet t = with(trie(), "ok");

        assertThatThrownBy(() -> t.insert(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.insert("")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.insert("   ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.insert("two words")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.insert("under_score")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.insert("café")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.insert("a.b")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.contains(" ")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.contains("a b")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.delete("")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.delete("a*")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.startsWith(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.startsWith("a b")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.countWithPrefix(null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.countWithPrefix("a!")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.wordsWithPrefix(null, 5)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.wordsWithPrefix("a/", 5)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRejectNonPositiveLimits() {
        WordSet t = with(trie(), "a");

        assertThatThrownBy(() -> t.wordsWithPrefix("a", 0)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.wordsWithPrefix("a", -1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.wordsWithPrefix("", 0)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldLeaveTheTrieUnchangedWhenACallIsRejected() {
        WordSet t = with(trie(), "alpha");

        assertThatThrownBy(() -> t.insert("al pha")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.insert("beta!")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> t.wordsWithPrefix("a", 0)).isInstanceOf(IllegalArgumentException.class);

        assertThat(t.size()).isEqualTo(1);
        assertThat(t.startsWith("b")).isFalse();
        assertThat(t.wordsWithPrefix("", 10)).containsExactly("alpha");
    }

    @Test
    void shouldCountWordsWithAPrefixIncludingTheWordItself() {
        WordSet t = with(trie(), "car", "card", "care", "careful", "cat", "dog");

        assertThat(t.countWithPrefix("car")).isEqualTo(4);
        assertThat(t.countWithPrefix("care")).isEqualTo(2);
        assertThat(t.countWithPrefix("ca")).isEqualTo(5);
        assertThat(t.countWithPrefix("c")).isEqualTo(5);
        assertThat(t.countWithPrefix("dogs")).isZero();
        assertThat(t.countWithPrefix("")).isEqualTo(6);
    }

    @Test
    void shouldListWordsInLexicographicOrderRegardlessOfInsertionOrder() {
        WordSet t = with(trie(), "pear", "peach", "pea", "pecan", "apple", "pe", "p2", "p-x");

        // plain character order: '-' < digits < letters
        assertThat(t.wordsWithPrefix("p", 20))
                .containsExactly("p-x", "p2", "pe", "pea", "peach", "pear", "pecan");
        assertThat(t.wordsWithPrefix("pea", 20)).containsExactly("pea", "peach", "pear");
    }

    @Test
    void shouldApplyTheLimitToTheSmallestWordsFirst() {
        WordSet t = with(trie(), "c", "a", "d", "b", "e");

        assertThat(t.wordsWithPrefix("", 3)).containsExactly("a", "b", "c");
        assertThat(t.wordsWithPrefix("", 1)).containsExactly("a");
        assertThat(t.wordsWithPrefix("", 100)).containsExactly("a", "b", "c", "d", "e");
    }

    @Test
    void shouldReturnAnEmptyListWhenNothingMatches() {
        WordSet t = with(trie(), "alpha");

        assertThat(t.wordsWithPrefix("b", 5)).isEmpty();
        assertThat(t.wordsWithPrefix("alphabet", 5)).isEmpty();
    }

    @Test
    void shouldReturnAFreshListTheCallerMayModify() {
        WordSet t = with(trie(), "a", "b");

        List<String> first = t.wordsWithPrefix("", 10);
        first.add("zzz");
        first.remove("a");

        assertThat(t.wordsWithPrefix("", 10)).containsExactly("a", "b");
        assertThat(t.size()).isEqualTo(2);
    }

    @Test
    void shouldDeleteOnlyTheGivenWordAndKeepWordsSharingItsPrefix() {
        WordSet t = with(trie(), "app", "apple", "apply", "ape");

        assertThat(t.delete("app")).isTrue();

        assertThat(t.contains("app")).isFalse();
        assertThat(t.contains("apple")).isTrue();
        assertThat(t.contains("apply")).isTrue();
        assertThat(t.startsWith("app")).isTrue();
        assertThat(t.wordsWithPrefix("ap", 10)).containsExactly("ape", "apple", "apply");
        assertThat(t.size()).isEqualTo(3);
    }

    @Test
    void shouldKeepTheShorterWordWhenALongerOneIsDeleted() {
        WordSet t = with(trie(), "app", "apple");

        assertThat(t.delete("apple")).isTrue();

        assertThat(t.contains("app")).isTrue();
        assertThat(t.startsWith("appl")).isFalse();
        assertThat(t.countWithPrefix("app")).isEqualTo(1);
        assertThat(t.wordsWithPrefix("", 10)).containsExactly("app");
    }

    @Test
    void shouldReturnFalseWhenDeletingAMissingWordOrOnlyAPrefix() {
        WordSet t = with(trie(), "apple");

        assertThat(t.delete("banana")).isFalse();
        assertThat(t.delete("app")).isFalse();
        assertThat(t.delete("apple")).isTrue();
        assertThat(t.delete("apple")).isFalse();
        assertThat(t.size()).isZero();
        assertThat(t.startsWith("a")).isFalse();
    }

    @Test
    void shouldAllowAWordToBeReinsertedAfterDeletion() {
        WordSet t = with(trie(), "tea");

        assertThat(t.delete("tea")).isTrue();
        assertThat(t.insert("tea")).isTrue();

        assertThat(t.contains("tea")).isTrue();
        assertThat(t.size()).isEqualTo(1);
    }

    @Test
    void shouldCountDistinctWordsInSize() {
        WordSet t = trie();
        assertThat(t.size()).isZero();

        with(t, "a", "b", "a", "ab", "B", " ab ");
        assertThat(t.size()).isEqualTo(3);

        t.delete("a");
        t.delete("zzz");
        assertThat(t.size()).isEqualTo(2);
    }

    @Test
    void shouldHandleVeryLongWords() {
        WordSet t = trie();
        String longWord = "a".repeat(1_000) + "-9";

        assertThat(t.insert(longWord)).isTrue();
        assertThat(t.contains(longWord)).isTrue();
        assertThat(t.startsWith("a".repeat(999))).isTrue();
        assertThat(t.countWithPrefix("a")).isEqualTo(1);
        assertThat(t.delete(longWord)).isTrue();
        assertThat(t.startsWith("a")).isFalse();
    }

    // ---------------------------------------------------------------- concurrency

    @Test
    void shouldLoseNoWordsWhenManyThreadsInsertDisjointWords() {
        WordSet t = trie();
        int threads = 8;
        int perThread = 5_000;

        Concurrently.run(threads, id -> {
            for (int i = 0; i < perThread; i++) {
                assertThat(t.insert(word(id * perThread + i))).isTrue();
            }
        });

        assertThat(t.size()).isEqualTo(threads * perThread);
        for (int i = 0; i < threads * perThread; i += 7) {
            assertThat(t.contains(word(i))).isTrue();
        }
    }

    @Test
    void shouldReturnTrueForExactlyOneThreadWhenTheSameWordIsInsertedConcurrently() {
        WordSet t = trie();

        List<Integer> wins = Concurrently.collect(16, id -> {
            int won = 0;
            for (int i = 0; i < 500; i++) {
                if (t.insert("shared-" + i)) {
                    won++;
                }
            }
            return won;
        });

        assertThat(wins.stream().mapToInt(Integer::intValue).sum()).isEqualTo(500);
        assertThat(t.size()).isEqualTo(500);
    }

    @Test
    void shouldReturnTrueForExactlyOneThreadWhenTheSameWordIsDeletedConcurrently() {
        WordSet t = trie();
        for (int i = 0; i < 500; i++) {
            t.insert("shared-" + i);
        }

        List<Integer> wins = Concurrently.collect(16, id -> {
            int won = 0;
            for (int i = 0; i < 500; i++) {
                if (t.delete("shared-" + i)) {
                    won++;
                }
            }
            return won;
        });

        assertThat(wins.stream().mapToInt(Integer::intValue).sum()).isEqualTo(500);
        assertThat(t.size()).isZero();
        assertThat(t.startsWith("")).isFalse();
    }

    @Test
    void shouldCountEachDistinctWordOnceWhenWritersInsertOverlappingRanges() {
        WordSet t = trie();

        Concurrently.run(8, id -> {
            for (int i = id * 500; i < id * 500 + 2_000; i++) {
                t.insert(word(i));
            }
        });

        // ranges [0,2000) .. [3500,5500) overlap: union is [0, 5500)
        assertThat(t.size()).isEqualTo(5_500);
        assertThat(t.countWithPrefix("")).isEqualTo(5_500);
        assertThat(t.wordsWithPrefix("", 10_000)).hasSize(5_500).doesNotHaveDuplicates();
    }

    @Test
    void shouldNotLoseEitherSideWhenInsertsAndDeletesOfDisjointWordsRunTogether() {
        WordSet t = trie();
        for (int i = 0; i < 4_000; i++) {
            t.insert("old" + word(i));
        }

        Concurrently.run(8, id -> {
            if (id < 4) {
                for (int i = id; i < 4_000; i += 4) {
                    assertThat(t.delete("old" + word(i))).isTrue();
                }
            } else {
                for (int i = id - 4; i < 4_000; i += 4) {
                    assertThat(t.insert("new" + word(i))).isTrue();
                }
            }
        });

        assertThat(t.size()).isEqualTo(4_000);
        assertThat(t.startsWith("old")).isFalse();
        assertThat(t.countWithPrefix("new")).isEqualTo(4_000);
    }

    @Test
    void shouldEndWithTheRightWordsWhenEveryThreadInsertsAndDeletesItsOwnWords() {
        WordSet t = trie();

        Concurrently.run(8, id -> {
            for (int round = 0; round < 200; round++) {
                for (int i = 0; i < 20; i++) {
                    assertThat(t.insert("t" + id + "-" + i)).isTrue();
                }
                for (int i = 0; i < 20; i += 2) {
                    assertThat(t.delete("t" + id + "-" + i)).isTrue();
                }
                for (int i = 1; i < 20; i += 2) {
                    assertThat(t.delete("t" + id + "-" + i)).isTrue();
                }
            }
            t.insert("t" + id + "-keep");
        });

        assertThat(t.size()).isEqualTo(8);
        assertThat(t.wordsWithPrefix("", 100))
                .containsExactly(
                        "t0-keep", "t1-keep", "t2-keep", "t3-keep", "t4-keep", "t5-keep", "t6-keep", "t7-keep");
    }

    @Test
    void shouldNeverShowAHalfInsertedWordToReaders() {
        WordSet t = trie();
        int writers = 4;
        int perWriter = 2_000;
        AtomicBoolean done = new AtomicBoolean();
        AtomicInteger writersLeft = new AtomicInteger(writers);

        Concurrently.run(writers + 4, id -> {
            if (id < writers) {
                for (int i = 0; i < perWriter; i++) {
                    t.insert("w" + id + "-" + word(i));
                }
                if (writersLeft.decrementAndGet() == 0) {
                    done.set(true);
                }
                return;
            }
            // Words are only ever added here, so once a word is seen it must stay fully visible.
            while (!done.get()) {
                for (int w = 0; w < writers; w++) {
                    for (int i = 0; i < perWriter; i += 37) {
                        String full = "w" + w + "-" + word(i);
                        if (t.contains(full)) {
                            assertThat(t.startsWith(full)).isTrue();
                            assertThat(t.startsWith(full.substring(0, full.length() - 1))).isTrue();
                            assertThat(t.countWithPrefix(full)).isGreaterThanOrEqualTo(1);
                            assertThat(t.wordsWithPrefix(full, 1)).containsExactly(full);
                        }
                    }
                }
            }
        });

        assertThat(t.size()).isEqualTo(writers * perWriter);
    }

    @Test
    void shouldReturnConsistentSortedSnapshotsWhileWordsAreInsertedAndDeleted() {
        WordSet t = trie();
        Set<String> universe = new HashSet<>();
        List<String> stable = new ArrayList<>();
        for (int i = 0; i < 200; i++) {
            stable.add("s" + word(i));
            t.insert("s" + word(i));
            universe.add("s" + word(i));
        }
        for (int i = 0; i < 1_000; i++) {
            universe.add("s-churn" + word(i));
        }
        AtomicInteger writersLeft = new AtomicInteger(2);
        AtomicBoolean done = new AtomicBoolean();

        Concurrently.run(6, id -> {
            if (id < 2) {
                for (int round = 0; round < 20; round++) {
                    for (int i = id; i < 1_000; i += 2) {
                        t.insert("s-churn" + word(i));
                    }
                    for (int i = id; i < 1_000; i += 2) {
                        t.delete("s-churn" + word(i));
                    }
                }
                if (writersLeft.decrementAndGet() == 0) {
                    done.set(true);
                }
                return;
            }
            while (!done.get()) {
                List<String> snapshot = t.wordsWithPrefix("s", 100_000);
                assertThat(snapshot).isSortedAccordingTo(String::compareTo).doesNotHaveDuplicates();
                assertThat(snapshot).isSubsetOf(universe);
                assertThat(snapshot).containsAll(stable);
                assertThat(t.countWithPrefix("s")).isBetween(200, 1_200);
            }
        });

        assertThat(t.size()).isEqualTo(200);
        assertThat(t.wordsWithPrefix("", 1_000)).hasSize(200).containsExactlyInAnyOrderElementsOf(stable);
    }

    @Test
    void shouldKeepSizeEqualToTheNumberOfListedWordsAfterMixedConcurrentWrites() {
        WordSet t = trie();

        Concurrently.run(8, id -> {
            for (int i = 0; i < 3_000; i++) {
                int k = (i * 7 + id * 13) % 1_500;
                if ((i + id) % 3 == 0) {
                    t.delete(word(k));
                } else {
                    t.insert(word(k));
                }
            }
        });

        List<String> all = t.wordsWithPrefix("", 100_000);
        assertThat(all).doesNotHaveDuplicates().isSortedAccordingTo(String::compareTo);
        assertThat(t.size()).isEqualTo(all.size());
        assertThat(t.countWithPrefix("")).isEqualTo(all.size());
        for (String w : all) {
            assertThat(t.contains(w)).isTrue();
        }
    }

    // ---------------------------------------------------------------- scale

    @Test
    void shouldHandleHundredThousandWordsAndTenThousandPrefixQueriesInReasonableTime() {
        WordSet t = trie();
        long start = System.nanoTime();

        for (int i = 0; i < 100_000; i++) {
            t.insert(word(i));
        }
        assertThat(t.size()).isEqualTo(100_000);

        int hits = 0;
        for (int i = 0; i < 10_000; i++) {
            String p = word(i * 7);
            if (t.startsWith(p)) {
                hits++;
            }
            assertThat(t.countWithPrefix(p)).isGreaterThanOrEqualTo(1);
            assertThat(t.wordsWithPrefix(p, 5)).isNotEmpty();
        }
        assertThat(hits).isEqualTo(10_000);
        for (int i = 0; i < 100_000; i += 2) {
            assertThat(t.delete(word(i))).isTrue();
        }
        assertThat(t.size()).isEqualTo(50_000);

        long elapsedMs = (System.nanoTime() - start) / 1_000_000;
        assertThat(elapsedMs).as("elapsed ms for 100k inserts, 10k prefix queries and 50k deletes").isLessThan(20_000);
    }
}

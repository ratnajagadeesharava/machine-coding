package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.DocumentSearch;
import java.util.ArrayList;
import java.util.List;
import java.util.concurrent.atomic.AtomicInteger;
import java.util.concurrent.atomic.AtomicReference;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private static DocumentSearch doc(String... lines) {
        return Candidate.of(() -> Solution.create(new ArrayList<>(List.of(lines))));
    }

    private static DocumentSearch large(int n) {
        List<String> lines = new ArrayList<>(n);
        for (int i = 1; i <= n; i++) {
            lines.add("line " + i + " group" + (i % 100) + " common Item" + (i % 7));
        }
        return Candidate.of(() -> Solution.create(lines));
    }

    @Test
    void shouldFindLinesContainingASingleWord() {
        DocumentSearch d = doc("The cat sat", "a dog barked", "cat and dog");

        assertThat(d.search("cat")).containsExactly(1, 3);
        assertThat(d.search("dog")).containsExactly(2, 3);
    }

    @Test
    void shouldAssignOneBasedIdsInDocumentOrder() {
        DocumentSearch d = doc("x", "y", "x", "y", "x");

        assertThat(d.search("x")).containsExactly(1, 3, 5);
        assertThat(d.search("y")).containsExactly(2, 4);
    }

    @Test
    void shouldRequireEveryWordOfTheQuery() {
        DocumentSearch d = doc("red green", "green blue", "red green blue", "red");

        assertThat(d.search("red green")).containsExactly(1, 3);
        assertThat(d.search("red green blue")).containsExactly(3);
        assertThat(d.search("red blue yellow")).isEmpty();
    }

    @Test
    void shouldIgnoreWordOrderAndPosition() {
        DocumentSearch d = doc("alpha beta gamma", "gamma x y z beta alpha", "beta alpha");

        assertThat(d.search("gamma alpha")).containsExactly(1, 2);
        assertThat(d.search("alpha gamma")).containsExactly(1, 2);
    }

    @Test
    void shouldMatchCaseInsensitively() {
        DocumentSearch d = doc("Hello World", "hello there", "HELLO");

        assertThat(d.search("hello")).containsExactly(1, 2, 3);
        assertThat(d.search("HeLLo wORLD")).containsExactly(1);
    }

    @Test
    void shouldMatchWholeWordsOnly() {
        DocumentSearch d = doc("A Category of cats", "concat bobcat", "the cat", "cat");

        assertThat(d.search("cat")).containsExactly(3, 4);
        assertThat(d.search("cats")).containsExactly(1);
        assertThat(d.search("cate")).isEmpty();
    }

    @Test
    void shouldTreatPunctuationAndUnderscoresAsSeparators() {
        DocumentSearch d = doc("cat, dog.", "(cat)", "the-cat", "cat_food", "tabs\tand\nnewline cat", "concat");

        assertThat(d.search("cat")).containsExactly(1, 2, 3, 4, 5);
        assertThat(d.search("food cat")).containsExactly(4);
        assertThat(d.search("dog")).containsExactly(1);
    }

    @Test
    void shouldTreatLettersAndDigitsAsOneWord() {
        DocumentSearch d = doc("v2 release notes", "release 2 notes", "version2");

        assertThat(d.search("v2")).containsExactly(1);
        assertThat(d.search("2")).containsExactly(2);
        assertThat(d.search("v")).isEmpty();
    }

    @Test
    void shouldSplitTheQueryWithTheSameRuleAsTheLines() {
        DocumentSearch d = doc("snake_case name", "snake only", "case only");

        assertThat(d.search("snake_case")).containsExactly(1);
        assertThat(d.search("  snake,   case!! ")).containsExactly(1);
    }

    @Test
    void shouldIgnoreDuplicateQueryWordsAndRepeatedWordsInALine() {
        DocumentSearch d = doc("cat cat cat", "cat dog", "dog");

        assertThat(d.search("cat cat")).containsExactly(1, 2);
        assertThat(d.search("cat dog cat dog")).containsExactly(2);
        assertThat(d.search("CAT cat Cat")).containsExactly(1, 2);
    }

    @Test
    void shouldReturnAnEmptyListForAQueryWithoutWords() {
        DocumentSearch d = doc("some text", "", "more text");

        assertThat(d.search("")).isEmpty();
        assertThat(d.search("   ")).isEmpty();
        assertThat(d.search("?! ,.")).isEmpty();
        assertThat(d.search("\t\n")).isEmpty();
    }

    @Test
    void shouldRejectANullQuery() {
        DocumentSearch d = doc("a");

        assertThatThrownBy(() -> d.search(null)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldRejectANullListOrANullLine() {
        doc("a"); // skips while unimplemented

        assertThatThrownBy(() -> Solution.create(null)).isInstanceOf(IllegalArgumentException.class);
        List<String> withNull = new ArrayList<>();
        withNull.add("a");
        withNull.add(null);
        assertThatThrownBy(() -> Solution.create(withNull)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldReturnAnEmptyListWhenNothingMatches() {
        DocumentSearch d = doc("one two", "three");

        assertThat(d.search("four")).isEmpty();
        assertThat(d.search("one four")).isEmpty();
    }

    @Test
    void shouldHandleAnEmptyDocumentAndBlankLines() {
        DocumentSearch empty = Candidate.of(() -> Solution.create(List.of()));
        assertThat(empty.search("a")).isEmpty();
        assertThat(empty.deleteLine(1)).isFalse();

        DocumentSearch d = doc("", "   ", "word", "");
        assertThat(d.search("word")).containsExactly(3);
        assertThat(d.deleteLine(1)).isTrue();
        assertThat(d.deleteLine(4)).isTrue();
        assertThat(d.search("word")).containsExactly(3);
    }

    @Test
    void shouldRemoveADeletedLineFromEverySearch() {
        DocumentSearch d = doc("cat dog", "cat", "dog cat bird");

        assertThat(d.deleteLine(1)).isTrue();

        assertThat(d.search("cat")).containsExactly(2, 3);
        assertThat(d.search("cat dog")).containsExactly(3);
        assertThat(d.search("dog")).containsExactly(3);
    }

    @Test
    void shouldKeepTheIdsOfTheOtherLinesAfterADeletion() {
        DocumentSearch d = doc("keep", "drop", "keep", "keep");

        assertThat(d.deleteLine(1)).isTrue();
        assertThat(d.search("keep")).containsExactly(3, 4);
        assertThat(d.deleteLine(4)).isTrue();
        assertThat(d.search("keep")).containsExactly(3);
        assertThat(d.search("drop")).containsExactly(2);
        assertThat(d.deleteLine(2)).isTrue();
        assertThat(d.search("drop")).isEmpty();
    }

    @Test
    void shouldReturnFalseForUnknownAndAlreadyDeletedIds() {
        DocumentSearch d = doc("a", "b", "c");

        assertThat(d.deleteLine(0)).isFalse();
        assertThat(d.deleteLine(-1)).isFalse();
        assertThat(d.deleteLine(4)).isFalse();
        assertThat(d.deleteLine(Integer.MAX_VALUE)).isFalse();
        assertThat(d.deleteLine(2)).isTrue();
        assertThat(d.deleteLine(2)).isFalse();
        assertThat(d.search("a")).containsExactly(1);
        assertThat(d.search("c")).containsExactly(3);
    }

    @Test
    void shouldNeverReuseTheIdOfADeletedLastLine() {
        DocumentSearch d = doc("a", "b");

        assertThat(d.deleteLine(2)).isTrue();
        assertThat(d.deleteLine(2)).isFalse();
        assertThat(d.deleteLine(3)).isFalse();
        assertThat(d.search("b")).isEmpty();
    }

    @Test
    void shouldReturnIdsInIncreasingOrderWithoutDuplicates() {
        DocumentSearch d = large(2_000);

        List<Integer> ids = d.search("common item3 group3");
        assertThat(ids).isNotEmpty().doesNotHaveDuplicates().isSorted();
        for (int id : ids) {
            assertThat(id % 7).isEqualTo(3);
            assertThat(id % 100).isEqualTo(3);
        }
    }

    @Test
    void shouldReturnACopyThatTheCallerMayModify() {
        DocumentSearch d = doc("x", "x", "x");

        List<Integer> first = d.search("x");
        try {
            first.clear();
        } catch (UnsupportedOperationException ignored) {
            // an unmodifiable list is also a safe copy
        }

        assertThat(d.search("x")).containsExactly(1, 2, 3);
    }

    @Test
    void shouldNotBeAffectedByLaterChangesToTheInputList() {
        List<String> source = new ArrayList<>(List.of("first line", "second line"));
        DocumentSearch d = Candidate.of(() -> Solution.create(source));

        source.set(0, "changed");
        source.add("third line");

        assertThat(d.search("first")).containsExactly(1);
        assertThat(d.search("changed")).isEmpty();
        assertThat(d.search("line")).containsExactly(1, 2);
    }

    @Test
    void shouldAnswerQueriesOnALargeDocument() {
        DocumentSearch d = large(20_000);

        assertThat(d.search("common")).hasSize(20_000);
        assertThat(d.search("group7")).hasSize(200);
        assertThat(d.search("group7 item3")).isNotEmpty().allMatch(id -> id % 100 == 7 && id % 7 == 3);
        assertThat(d.search("line 20000")).containsExactly(20_000);
        assertThat(d.search("group7 group8")).isEmpty();
        assertThat(d.search("group7 nosuchword")).isEmpty();
    }

    @Test
    void shouldAnswerThousandsOfSearchesOnALargeDocumentQuickly() {
        DocumentSearch d = large(20_000);

        long start = System.nanoTime();
        long total = 0;
        for (int i = 0; i < 2_000; i++) {
            total += d.search("group" + (i % 100) + " item" + (i % 7)).size();
        }
        long elapsedMs = (System.nanoTime() - start) / 1_000_000;

        assertThat(total).isGreaterThan(0);
        assertThat(elapsedMs).as("2,000 searches over 20,000 lines").isLessThan(10_000);
    }

    @Test
    void shouldNotCountDeletedLinesInALargeDocument() {
        DocumentSearch d = large(20_000);

        for (int id = 2; id <= 20_000; id += 2) {
            assertThat(d.deleteLine(id)).isTrue();
        }

        assertThat(d.search("common")).hasSize(10_000).allMatch(id -> id % 2 == 1);
        assertThat(d.search("line 3")).containsExactly(3);
        assertThat(d.search("line 4")).isEmpty();
    }

    @Test
    void shouldLetEightThreadsDeleteDifferentLinesWithoutLosingAny() {
        DocumentSearch d = large(8_000);

        List<Integer> trues = Concurrently.collect(8, t -> {
            int deleted = 0;
            for (int id = t + 1; id <= 8_000; id += 8) {
                if (d.deleteLine(id)) {
                    deleted++;
                }
            }
            return deleted;
        });

        assertThat(trues).containsOnly(1_000);
        assertThat(d.search("common")).isEmpty();
    }

    @Test
    void shouldTellExactlyOneOfManyRacingThreadsThatItDeletedTheLine() {
        DocumentSearch d = doc("target", "other");
        AtomicInteger winners = new AtomicInteger();

        Concurrently.run(16, t -> {
            if (d.deleteLine(1)) {
                winners.incrementAndGet();
            }
        });

        assertThat(winners.get()).isEqualTo(1);
        assertThat(d.search("target")).isEmpty();
        assertThat(d.search("other")).containsExactly(2);
    }

    @Test
    void shouldSearchWhileLinesAreBeingDeleted() {
        int n = 4_000;
        DocumentSearch d = large(n);
        AtomicInteger deletedUpTo = new AtomicInteger(0);
        AtomicReference<String> problem = new AtomicReference<>();

        Concurrently.run(5, t -> {
            if (t == 0) {
                for (int id = 1; id <= n; id++) {
                    d.deleteLine(id);
                    deletedUpTo.set(id);
                }
                return;
            }
            for (int i = 0; i < 300; i++) {
                int floor = deletedUpTo.get();
                List<Integer> ids = d.search("common");
                for (int k = 0; k < ids.size(); k++) {
                    if (ids.get(k) <= floor) {
                        problem.compareAndSet(null, "deleted id " + ids.get(k) + " returned, floor " + floor);
                    }
                    if (k > 0 && ids.get(k - 1) >= ids.get(k)) {
                        problem.compareAndSet(null, "result not increasing");
                    }
                }
            }
        });

        assertThat(problem.get()).isNull();
        assertThat(d.search("common")).isEmpty();
    }
}

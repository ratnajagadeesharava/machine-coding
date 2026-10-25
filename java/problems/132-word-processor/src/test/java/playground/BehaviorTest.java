package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.WordProcessor;
import java.util.List;
import java.util.Map;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;
import testkit.Concurrently;

class BehaviorTest {

    private static final String DEF = "Calibri|11|-|-|-|black";

    private WordProcessor doc() {
        return Candidate.of(Solution::create);
    }

    private WordProcessor doc(String text) {
        WordProcessor d = doc();
        d.insert(0, text);
        return d;
    }

    @Test
    void shouldStartEmpty() {
        WordProcessor d = doc();

        assertThat(d.text()).isEmpty();
        assertThat(d.length()).isZero();
        assertThat(d.runs()).isEmpty();
        assertThat(d.wordCount()).isZero();
        assertThat(d.lineCount()).isEqualTo(1);
        assertThat(d.alignment(0)).isEqualTo("LEFT");
        assertThat(d.distinctFormatCount()).isZero();
    }

    @Test
    void shouldGiveInsertedTextTheDefaultFormatInAnEmptyDocument() {
        WordProcessor d = doc();
        d.insert(0, "hello");

        assertThat(d.text()).isEqualTo("hello");
        assertThat(d.length()).isEqualTo(5);
        assertThat(d.formatAt(0)).isEqualTo(DEF);
        assertThat(d.formatAt(4)).isEqualTo(DEF);
        assertThat(d.runs()).containsExactly("0-5:" + DEF);
    }

    @Test
    void shouldInsertAtTheStartInTheMiddleAndAtTheEnd() {
        WordProcessor d = doc("ace");
        d.insert(1, "b");
        d.insert(3, "d");
        d.insert(0, ">");
        d.insert(d.length(), "<");
        d.insert(2, "");

        assertThat(d.text()).isEqualTo(">abcde<");
    }

    @Test
    void shouldRejectInsertAtAnInvalidPosition() {
        WordProcessor d = doc("abc");

        assertThatThrownBy(() -> d.insert(4, "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.insert(-1, "x")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.insert(1, null)).isInstanceOf(IllegalArgumentException.class);
        assertThat(d.text()).isEqualTo("abc");
    }

    @Test
    void shouldGiveInsertedTextTheFormatOfThePrecedingCharacter() {
        WordProcessor d = doc("abcd");
        d.applyFormat(0, 2, Map.of("bold", "true", "color", "red"));
        d.applyFormat(2, 4, Map.of("italic", "true"));

        d.insert(1, "X");
        assertThat(d.formatAt(1)).isEqualTo("Calibri|11|B|-|-|red");
        d.insert(4, "Y");
        assertThat(d.formatAt(4)).isEqualTo(DEF.replace("-|-|-", "-|I|-"));
        d.insert(d.length(), "Z");
        assertThat(d.formatAt(d.length() - 1)).isEqualTo(DEF.replace("-|-|-", "-|I|-"));
        assertThat(d.text()).isEqualTo("aXbcYdZ");
    }

    @Test
    void shouldUseTheDefaultFormatWhenInsertingAtPositionZeroEvenIfTheFirstCharacterIsFormatted() {
        WordProcessor d = doc("abc");
        d.applyFormat(0, 3, Map.of("font", "Arial", "size", "20", "underline", "true"));

        d.insert(0, "X");

        assertThat(d.formatAt(0)).isEqualTo(DEF);
        assertThat(d.formatAt(1)).isEqualTo("Arial|20|-|-|U|black");
        assertThat(d.runs()).containsExactly("0-1:" + DEF, "1-4:Arial|20|-|-|U|black");
    }

    @Test
    void shouldUseTheDefaultFormatAgainAfterEverythingWasDeleted() {
        WordProcessor d = doc("abc");
        d.applyFormat(0, 3, Map.of("bold", "true"));
        d.delete(0, 3);
        assertThat(d.length()).isZero();
        assertThat(d.runs()).isEmpty();

        d.insert(0, "new");

        assertThat(d.runs()).containsExactly("0-3:" + DEF);
    }

    @Test
    void shouldDeleteAHalfOpenRange() {
        WordProcessor d = doc("0123456789");
        d.delete(2, 5);
        assertThat(d.text()).isEqualTo("0156789");
        d.delete(0, 1);
        assertThat(d.text()).isEqualTo("156789");
        d.delete(4, 6);
        assertThat(d.text()).isEqualTo("1567");
        d.delete(2, 2);
        assertThat(d.text()).isEqualTo("1567");
        d.delete(0, 4);
        assertThat(d.text()).isEmpty();
    }

    @Test
    void shouldRejectInvalidDeleteRanges() {
        WordProcessor d = doc("abc");

        assertThatThrownBy(() -> d.delete(2, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.delete(-1, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.delete(1, 4)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.delete(4, 4)).isInstanceOf(IllegalArgumentException.class);
        assertThat(d.text()).isEqualTo("abc");
    }

    @Test
    void shouldChangeOnlyTheFieldsThatAreGiven() {
        WordProcessor d = doc("abcdef");
        d.applyFormat(1, 4, Map.of("font", "Arial", "size", "14"));
        d.applyFormat(2, 3, Map.of("italic", "true"));

        assertThat(d.formatAt(0)).isEqualTo(DEF);
        assertThat(d.formatAt(1)).isEqualTo("Arial|14|-|-|-|black");
        assertThat(d.formatAt(2)).isEqualTo("Arial|14|-|I|-|black");
        assertThat(d.formatAt(3)).isEqualTo("Arial|14|-|-|-|black");
        assertThat(d.formatAt(4)).isEqualTo(DEF);
    }

    @Test
    void shouldShowEveryFlagInTheFormatString() {
        WordProcessor d = doc("x");
        d.applyFormat(0, 1, Map.of("font", "Times New Roman", "size", "72", "bold", "true", "italic", "true",
                "underline", "true", "color", "dark red"));

        assertThat(d.formatAt(0)).isEqualTo("Times New Roman|72|B|I|U|dark red");

        d.applyFormat(0, 1, Map.of("bold", "false", "underline", "false"));
        assertThat(d.formatAt(0)).isEqualTo("Times New Roman|72|-|I|-|dark red");
    }

    @Test
    void shouldRejectAnInvalidFormatChangeWithoutApplyingAnyPartOfIt() {
        WordProcessor d = doc("abcdef");

        assertThatThrownBy(() -> d.applyFormat(0, 3, Map.of("bold", "true", "size", "0")))
                .isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.applyFormat(0, 3, Map.of("size", "1001"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.applyFormat(0, 3, Map.of("size", "big"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.applyFormat(0, 3, Map.of("bold", "yes"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.applyFormat(0, 3, Map.of("font", " "))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.applyFormat(0, 3, Map.of("color", "a|b"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.applyFormat(0, 3, Map.of("strike", "true"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.applyFormat(0, 7, Map.of("bold", "true"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.applyFormat(4, 2, Map.of("bold", "true"))).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.applyFormat(-1, 2, Map.of("bold", "true"))).isInstanceOf(IllegalArgumentException.class);

        assertThat(d.runs()).containsExactly("0-6:" + DEF);
    }

    @Test
    void shouldTreatAnEmptyChangeOrAnEmptyRangeAsANoOp() {
        WordProcessor d = doc("abc");
        d.applyFormat(0, 3, Map.of());
        d.applyFormat(2, 2, Map.of("bold", "true"));
        d.applyFormat(3, 3, Map.of("bold", "true"));

        assertThat(d.runs()).containsExactly("0-3:" + DEF);
    }

    @Test
    void shouldRejectFormatAtOutsideTheText() {
        WordProcessor d = doc("abc");

        assertThatThrownBy(() -> d.formatAt(3)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.formatAt(-1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> doc().formatAt(0)).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldReportMaximalRunsWithExclusiveEnds() {
        WordProcessor d = doc("0123456789");
        d.applyFormat(2, 5, Map.of("bold", "true"));
        d.applyFormat(7, 9, Map.of("color", "red"));

        assertThat(d.runs()).containsExactly(
                "0-2:" + DEF,
                "2-5:Calibri|11|B|-|-|black",
                "5-7:" + DEF,
                "7-9:Calibri|11|-|-|-|red",
                "9-10:" + DEF);
    }

    @Test
    void shouldMergeAdjacentRunsThatBecomeEqual() {
        WordProcessor d = doc("0123456789");
        d.applyFormat(0, 5, Map.of("bold", "true"));
        d.applyFormat(5, 10, Map.of("bold", "true"));
        assertThat(d.runs()).containsExactly("0-10:Calibri|11|B|-|-|black");

        d.applyFormat(3, 6, Map.of("bold", "false"));
        assertThat(d.runs()).hasSize(3);

        d.applyFormat(0, 10, Map.of("bold", "false"));
        assertThat(d.runs()).containsExactly("0-10:" + DEF);
    }

    @Test
    void shouldMergeRunsAcrossADeletedMiddle() {
        WordProcessor d = doc("aaXbb");
        d.applyFormat(2, 3, Map.of("underline", "true"));
        assertThat(d.runs()).hasSize(3);

        d.delete(2, 3);

        assertThat(d.text()).isEqualTo("aabb");
        assertThat(d.runs()).containsExactly("0-4:" + DEF);
    }

    @Test
    void shouldKeepASingleRunWhenInsertingInsideAFormattedRun() {
        WordProcessor d = doc("abcdef");
        d.applyFormat(2, 5, Map.of("bold", "true"));

        d.insert(3, "ZZ");

        assertThat(d.runs()).containsExactly("0-2:" + DEF, "2-7:Calibri|11|B|-|-|black", "7-8:" + DEF);
    }

    @Test
    void shouldFindOverlappingMatchesInAscendingOrder() {
        WordProcessor d = doc("aaaa abab");

        assertThat(d.find("aa", true)).containsExactly(0, 1, 2);
        assertThat(d.find("aba", true)).containsExactly(5);
        assertThat(d.find("ab", true)).containsExactly(5, 7);
        assertThat(d.find("zz", true)).isEmpty();
        assertThat(d.find("aaaa abab", true)).containsExactly(0);
        assertThat(d.find("aaaa abab!", true)).isEmpty();
    }

    @Test
    void shouldFindIgnoringCaseOnlyWhenAsked() {
        WordProcessor d = doc("Cat cAT CAT cat");

        assertThat(d.find("cat", true)).containsExactly(12);
        assertThat(d.find("cat", false)).containsExactly(0, 4, 8, 12);
        assertThat(d.find("CAT", false)).containsExactly(0, 4, 8, 12);
    }

    @Test
    void shouldFindAcrossFormatBoundaries() {
        WordProcessor d = doc("hello world");
        d.applyFormat(3, 8, Map.of("bold", "true"));

        assertThat(d.find("lo wo", true)).containsExactly(3);
    }

    @Test
    void shouldRejectAnEmptyPattern() {
        WordProcessor d = doc("abc");

        assertThatThrownBy(() -> d.find("", true)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.find(null, true)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.replaceAll("", "x", true)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.replaceAll("a", null, true)).isInstanceOf(IllegalArgumentException.class);
        assertThat(d.text()).isEqualTo("abc");
    }

    @Test
    void shouldReplaceAllNonOverlappingMatchesAndReturnTheCount() {
        WordProcessor d = doc("the cat and the hat");

        assertThat(d.replaceAll("the", "a", true)).isEqualTo(2);
        assertThat(d.text()).isEqualTo("a cat and a hat");
        assertThat(d.replaceAll("zzz", "q", true)).isZero();
        assertThat(d.text()).isEqualTo("a cat and a hat");

        WordProcessor e = doc("aaaa");
        assertThat(e.replaceAll("aa", "b", true)).isEqualTo(2);
        assertThat(e.text()).isEqualTo("bb");

        WordProcessor g = doc("aaa");
        assertThat(g.replaceAll("aa", "b", true)).isEqualTo(1);
        assertThat(g.text()).isEqualTo("ba");
    }

    @Test
    void shouldNotScanTheReplacementTextAgain() {
        WordProcessor d = doc("ab ab");

        assertThat(d.replaceAll("ab", "abab", true)).isEqualTo(2);
        assertThat(d.text()).isEqualTo("abab abab");
    }

    @Test
    void shouldReplaceIgnoringCaseWhenAsked() {
        WordProcessor d = doc("Foo foo FOO");

        assertThat(d.replaceAll("foo", "x", true)).isEqualTo(1);
        assertThat(d.text()).isEqualTo("Foo x FOO");
        assertThat(d.replaceAll("foo", "y", false)).isEqualTo(2);
        assertThat(d.text()).isEqualTo("y x y");
    }

    @Test
    void shouldDeleteMatchesWhenTheReplacementIsEmpty() {
        WordProcessor d = doc("a-b-c");

        assertThat(d.replaceAll("-", "", true)).isEqualTo(2);
        assertThat(d.text()).isEqualTo("abc");
        assertThat(d.runs()).containsExactly("0-3:" + DEF);
    }

    @Test
    void shouldGiveReplacementTextTheFormatOfTheFirstCharacterOfEachMatch() {
        WordProcessor d = doc("one two one");
        d.applyFormat(0, 1, Map.of("bold", "true"));
        d.applyFormat(1, 3, Map.of("italic", "true"));
        d.applyFormat(8, 11, Map.of("color", "red"));

        assertThat(d.replaceAll("one", "1234", true)).isEqualTo(2);

        assertThat(d.text()).isEqualTo("1234 two 1234");
        assertThat(d.runs()).containsExactly(
                "0-4:Calibri|11|B|-|-|black",
                "4-9:" + DEF,
                "9-13:Calibri|11|-|-|-|red");
    }

    @Test
    void shouldCountWordsAsMaximalNonWhitespaceSequences() {
        assertThat(doc().wordCount()).isZero();
        assertThat(doc("   \n\t ").wordCount()).isZero();
        assertThat(doc("hello").wordCount()).isEqualTo(1);
        assertThat(doc("  hello   world  ").wordCount()).isEqualTo(2);
        assertThat(doc("a\nb\tc\r\nd").wordCount()).isEqualTo(4);
        assertThat(doc("don't stop-me now.").wordCount()).isEqualTo(3);
    }

    @Test
    void shouldCountLinesAsNewlinesPlusOne() {
        assertThat(doc().lineCount()).isEqualTo(1);
        assertThat(doc("abc").lineCount()).isEqualTo(1);
        assertThat(doc("a\nb\nc").lineCount()).isEqualTo(3);
        assertThat(doc("a\n").lineCount()).isEqualTo(2);
        assertThat(doc("\n\n").lineCount()).isEqualTo(3);

        WordProcessor d = doc("a\nb\nc");
        d.delete(1, 4);
        assertThat(d.text()).isEqualTo("ac");
        assertThat(d.lineCount()).isEqualTo(1);
    }

    @Test
    void shouldSetAndReadParagraphAlignment() {
        WordProcessor d = doc("one\ntwo\nthree");

        assertThat(d.alignment(0)).isEqualTo("LEFT");
        d.setAlignment(1, "CENTER");
        d.setAlignment(2, "JUSTIFY");
        d.setAlignment(0, "RIGHT");

        assertThat(d.alignment(0)).isEqualTo("RIGHT");
        assertThat(d.alignment(1)).isEqualTo("CENTER");
        assertThat(d.alignment(2)).isEqualTo("JUSTIFY");
    }

    @Test
    void shouldRejectInvalidAlignmentRequests() {
        WordProcessor d = doc("one\ntwo");

        assertThatThrownBy(() -> d.setAlignment(2, "LEFT")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.setAlignment(-1, "LEFT")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.setAlignment(0, "left")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.setAlignment(0, "MIDDLE")).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.setAlignment(0, null)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.alignment(2)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> d.alignment(-1)).isInstanceOf(IllegalArgumentException.class);
        assertThat(d.alignment(0)).isEqualTo("LEFT");
    }

    @Test
    void shouldGiveNewLinesTheAlignmentOfTheLineTheyWereCreatedIn() {
        WordProcessor d = doc("one\ntwo");
        d.setAlignment(0, "CENTER");
        d.setAlignment(1, "RIGHT");

        d.insert(3, "\nnew");

        assertThat(d.text()).isEqualTo("one\nnew\ntwo");
        assertThat(d.alignment(0)).isEqualTo("CENTER");
        assertThat(d.alignment(1)).isEqualTo("CENTER");
        assertThat(d.alignment(2)).isEqualTo("RIGHT");

        d.insert(9, "x\ny\nz");
        assertThat(d.lineCount()).isEqualTo(5);
        assertThat(d.alignment(2)).isEqualTo("RIGHT");
        assertThat(d.alignment(3)).isEqualTo("RIGHT");
        assertThat(d.alignment(4)).isEqualTo("RIGHT");
    }

    @Test
    void shouldKeepTheAlignmentOfTheFirstLineWhenLinesAreMergedByDeleting() {
        WordProcessor d = doc("aa\nbb\ncc\ndd");
        d.setAlignment(0, "LEFT");
        d.setAlignment(1, "CENTER");
        d.setAlignment(2, "RIGHT");
        d.setAlignment(3, "JUSTIFY");

        d.delete(4, 7);

        assertThat(d.text()).isEqualTo("aa\nbc\ndd");
        assertThat(d.lineCount()).isEqualTo(3);
        assertThat(d.alignment(0)).isEqualTo("LEFT");
        assertThat(d.alignment(1)).isEqualTo("CENTER");
        assertThat(d.alignment(2)).isEqualTo("JUSTIFY");
    }

    @Test
    void shouldCountDistinctFormatsInUse() {
        WordProcessor d = doc("abcdef");
        assertThat(d.distinctFormatCount()).isEqualTo(1);

        d.applyFormat(0, 2, Map.of("bold", "true"));
        d.applyFormat(4, 6, Map.of("bold", "true"));
        assertThat(d.distinctFormatCount()).isEqualTo(2);

        d.applyFormat(1, 4, Map.of("color", "red"));
        assertThat(d.distinctFormatCount()).isEqualTo(3);

        d.applyFormat(0, 6, Map.of("bold", "false", "color", "black"));
        assertThat(d.distinctFormatCount()).isEqualTo(1);

        d.delete(0, 6);
        assertThat(d.distinctFormatCount()).isZero();
    }

    @Test
    void shouldLoseNoCharactersWhenThreadsInsertAtTheEnd() {
        WordProcessor d = doc();
        int threads = 8;
        int each = 1_000;

        Concurrently.run(threads, t -> {
            String ch = String.valueOf((char) ('a' + t));
            for (int i = 0; i < each; i++) {
                d.insert(d.length(), ch);
            }
        });

        assertThat(d.length()).isEqualTo(threads * each);
        String text = d.text();
        for (int t = 0; t < threads; t++) {
            char c = (char) ('a' + t);
            assertThat(text.chars().filter(x -> x == c).count()).isEqualTo(each);
        }
        assertThat(d.runs()).containsExactly("0-" + (threads * each) + ":" + DEF);
    }

    @Test
    void shouldStayConsistentWhileThreadsFormatAndRead() {
        WordProcessor d = doc("x".repeat(1_000));

        Concurrently.run(6, t -> {
            for (int i = 0; i < 300; i++) {
                if (t < 3) {
                    d.applyFormat(i, i + 100, Map.of("size", String.valueOf(10 + t)));
                } else {
                    List<String> runs = d.runs();
                    assertThat(runs.get(0)).startsWith("0-");
                    assertThat(runs.get(runs.size() - 1)).contains("-1000:");
                    assertThat(d.distinctFormatCount()).isBetween(1, 4);
                }
            }
        });

        assertThat(d.text()).isEqualTo("x".repeat(1_000));
    }

    @Test
    void shouldHandleAMillionCharactersInAFewRuns() {
        long start = System.nanoTime();
        WordProcessor d = doc();
        d.insert(0, "a".repeat(1_000_000));
        for (int i = 0; i < 10; i++) {
            d.applyFormat(i * 100_000 + 10_000, i * 100_000 + 20_000, Map.of("size", String.valueOf(12 + i)));
        }

        assertThat(d.length()).isEqualTo(1_000_000);
        assertThat(d.runs()).hasSize(21);
        assertThat(d.distinctFormatCount()).isEqualTo(11);
        assertThat(d.formatAt(15_000)).isEqualTo("Calibri|12|-|-|-|black");
        assertThat(d.formatAt(999_999)).isEqualTo(DEF);

        for (int i = 0; i < 2_000; i++) {
            d.insert(500_000, "b");
            d.formatAt(i * 400);
        }
        d.applyFormat(0, 1_000_000, Map.of("size", "11"));
        assertThat(d.runs()).containsExactly("0-1002000:" + DEF);
        assertThat(d.find("b", true)).hasSize(2_000);
        assertThat(d.wordCount()).isEqualTo(1);

        assertThat((System.nanoTime() - start) / 1_000_000).isLessThan(20_000);
    }
}

package playground;

import static org.assertj.core.api.Assertions.assertThat;
import static org.assertj.core.api.Assertions.assertThatThrownBy;

import contract.Editor;
import org.junit.jupiter.api.Test;
import solution.Solution;
import testkit.Candidate;

class BehaviorTest {

    private Editor editor() {
        return Candidate.of(Solution::create);
    }

    private static void state(Editor e, String text, int cursor) {
        assertThat(e.text()).isEqualTo(text);
        assertThat(e.cursor()).isEqualTo(cursor);
    }

    @Test
    void shouldInsertAtTheCursor() {
        Editor e = editor();

        e.type("hello");
        e.moveCursor(2);
        e.type("XY");

        state(e, "heXYllo", 4);
    }

    @Test
    void shouldBackspaceBeforeTheCursorAndStopAtTheStart() {
        Editor e = editor();
        e.type("abc");

        e.backspace();
        state(e, "ab", 2);
        e.moveCursor(0);
        e.backspace();
        state(e, "ab", 0);
    }

    @Test
    void shouldReplaceTheSelectionWhenTyping() {
        Editor e = editor();
        e.type("hello world");

        e.select(0, 5);
        assertThat(e.selectedText()).isEqualTo("hello");
        e.type("bye");

        state(e, "bye world", 3);
        assertThat(e.selectedText()).isEmpty();
    }

    @Test
    void shouldDeleteTheSelectionOnBackspace() {
        Editor e = editor();
        e.type("hello world");

        e.select(5, 11);
        e.backspace();

        state(e, "hello", 5);
    }

    @Test
    void shouldRejectOutOfRangePositions() {
        Editor e = editor();
        e.type("abc");

        assertThatThrownBy(() -> e.moveCursor(4)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.moveCursor(-1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.select(2, 1)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.select(0, 4)).isInstanceOf(IllegalArgumentException.class);
        assertThatThrownBy(() -> e.replaceAll("", "x")).isInstanceOf(IllegalArgumentException.class);
    }

    @Test
    void shouldCopyCutAndPaste() {
        Editor e = editor();
        e.type("abcdef");

        e.select(1, 3);
        e.copy();
        e.moveCursor(6);
        e.paste();
        state(e, "abcdefbc", 8);

        e.select(0, 2);
        e.cut();
        state(e, "cdefbc", 0);
        e.moveCursor(6);
        e.paste();
        state(e, "cdefbcab", 8);
    }

    @Test
    void shouldGroupConsecutiveTypingIntoOneUndoStep() {
        Editor e = editor();

        e.type("a");
        e.type("b");
        e.type("c");
        assertThat(e.undo()).isTrue();

        state(e, "", 0);
        assertThat(e.undo()).isFalse();
    }

    @Test
    void shouldEndTheTypingGroupWhenTheCursorMoves() {
        Editor e = editor();
        e.type("ab");
        e.moveCursor(0);
        e.type("X");
        e.type("Y");

        e.undo();
        state(e, "ab", 0);
        e.undo();
        state(e, "", 0);
    }

    @Test
    void shouldGroupConsecutiveBackspacesButNotTypingWithBackspace() {
        Editor e = editor();
        e.type("hello");
        e.backspace();
        e.backspace();
        e.type("p");

        e.undo(); // the "p"
        state(e, "hel", 3);
        e.undo(); // both backspaces together
        state(e, "hello", 5);
        e.undo(); // the typing
        state(e, "", 0);
    }

    @Test
    void shouldNotGroupTypingOverASelection() {
        Editor e = editor();
        e.type("hello");
        e.select(0, 5);
        e.type("a");
        e.select(0, 1);
        e.type("b");

        e.undo();
        state(e, "a", 1);
        e.undo();
        state(e, "hello", 5);
    }

    @Test
    void shouldRestoreTheCursorOnUndoAndRedo() {
        Editor e = editor();
        e.type("hello");
        e.moveCursor(1);
        e.type("XX");
        state(e, "hXXello", 3);

        e.undo();
        state(e, "hello", 1);
        e.redo();
        state(e, "hXXello", 3);
        assertThat(e.redo()).isFalse();
    }

    @Test
    void shouldClearTheRedoListOnANewEdit() {
        Editor e = editor();
        e.type("a");
        e.moveCursor(1);
        e.type("b");
        e.undo();

        e.type("c");

        assertThat(e.redo()).isFalse();
        state(e, "ac", 2);
    }

    @Test
    void shouldUndoCutAndPasteAsSingleSteps() {
        Editor e = editor();
        e.type("abcdef");
        e.select(2, 4);
        e.cut();
        e.paste();
        state(e, "abcdef", 4);

        e.undo();
        state(e, "abef", 2);
        e.undo();
        state(e, "abcdef", 4);
    }

    @Test
    void shouldReplaceAllNonOverlappingOccurrencesAsOneStep() {
        Editor e = editor();
        e.type("aaaa b aa");

        int count = e.replaceAll("aa", "X");

        assertThat(count).isEqualTo(3);
        assertThat(e.text()).isEqualTo("XX b X");
        e.undo();
        assertThat(e.text()).isEqualTo("aaaa b aa");
        e.redo();
        assertThat(e.text()).isEqualTo("XX b X");
        assertThat(e.replaceAll("zzz", "y")).isZero();
        e.undo(); // the zero-count replace made no step: this undoes the previous replace
        assertThat(e.text()).isEqualTo("aaaa b aa");
    }

    @Test
    void shouldIgnoreEmptyTypingAndSelectionOnlyOperationsForUndo() {
        Editor e = editor();
        e.type("abc");

        e.type("");
        e.select(0, 2);
        e.copy();
        e.moveCursor(1);

        e.undo();
        state(e, "", 0);
        assertThat(e.undo()).isFalse();
    }

    @Test
    void shouldSurviveLongSessionsOfUndoAndRedo() {
        Editor e = editor();
        for (int i = 0; i < 200; i++) {
            e.type("x");
            e.moveCursor(e.text().length()); // each moveCursor ends the group: 200 separate steps
        }

        for (int i = 0; i < 200; i++) {
            assertThat(e.undo()).isTrue();
        }
        assertThat(e.text()).isEmpty();
        for (int i = 0; i < 200; i++) {
            assertThat(e.redo()).isTrue();
        }
        assertThat(e.text()).hasSize(200);
    }
}

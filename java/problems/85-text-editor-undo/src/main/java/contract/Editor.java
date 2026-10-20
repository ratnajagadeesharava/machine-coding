package contract;

/** Thin test entry point. Your design lives behind it. Single-threaded use. */
public interface Editor {

    /**
     * Replaces the selection (if any) with {@code s}, otherwise inserts at the cursor. The cursor ends after the
     * inserted text and the selection is cleared. An empty string does nothing (and creates no undo step).
     */
    void type(String s);

    /** Deletes the selection, or the character before the cursor. Does nothing at position 0 without selection. */
    void backspace();

    /** Moves the cursor (clearing any selection). IllegalArgumentException unless 0 <= pos <= length. */
    void moveCursor(int pos);

    /** Selects [from, to) and puts the cursor at {@code to}. IllegalArgumentException unless 0 <= from <= to <= length. */
    void select(int from, int to);

    /** The selected text, or "" without a selection. */
    String selectedText();

    /** Copies the selection into the clipboard. Does nothing without a selection. */
    void copy();

    /** Copy + delete the selection (one undo step). Does nothing without a selection. */
    void cut();

    /** Same as type(clipboard) (one undo step). */
    void paste();

    /**
     * Replaces every non-overlapping occurrence of {@code find} (scanning left to right) and returns the count.
     * One undo step if the count is > 0. The cursor keeps its index, clamped to the new length; the selection is
     * cleared. IllegalArgumentException if {@code find} is empty.
     */
    int replaceAll(String find, String with);

    /**
     * Undoes the last step: text and cursor return to exactly what they were before it; the selection is cleared.
     * Returns false if there is nothing to undo.
     *
     * <p>Grouping: consecutive type() calls (each without a selection) form ONE step, and consecutive backspace()
     * calls (each without a selection) form ONE step. Any other call in between (moveCursor, select, copy, cut,
     * paste, replaceAll, undo, redo, or switching between type and backspace) ends the group.
     */
    boolean undo();

    /** Re-applies the last undone step (text and cursor as they were after it). False if nothing to redo. */
    boolean redo();

    String text();

    int cursor();
}

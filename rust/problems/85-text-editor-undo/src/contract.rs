//! Thin test entry point. Your design lives behind it. Single-threaded use (`&mut self`).
//!
//! Positions are counted in `char`s (the tests only use ASCII).

#[derive(Debug, PartialEq, Eq)]
pub enum EditorError {
    /// position/range outside the text, or an empty `find` string
    InvalidArgument,
}

pub trait Editor {
    /// Replaces the selection (if any) with `s`, otherwise inserts at the cursor. The cursor ends after the
    /// inserted text and the selection is cleared. An empty string does nothing (and creates no undo step).
    fn type_text(&mut self, s: &str);

    /// Deletes the selection, or the character before the cursor. Does nothing at position 0 without selection.
    fn backspace(&mut self);

    /// Moves the cursor (clearing any selection). Error unless `pos <= length`.
    fn move_cursor(&mut self, pos: usize) -> Result<(), EditorError>;

    /// Selects `[from, to)` and puts the cursor at `to`. Error unless `from <= to <= length`.
    fn select(&mut self, from: usize, to: usize) -> Result<(), EditorError>;

    /// The selected text, or "" without a selection.
    fn selected_text(&self) -> String;

    /// Copies the selection into the clipboard. Does nothing without a selection.
    fn copy(&mut self);

    /// Copy + delete the selection (one undo step). Does nothing without a selection.
    fn cut(&mut self);

    /// Same as `type_text(clipboard)` (one undo step).
    fn paste(&mut self);

    /// Replaces every non-overlapping occurrence of `find` (scanning left to right) and returns the count.
    /// One undo step if the count is > 0. The cursor keeps its index, clamped to the new length; the selection is
    /// cleared. Error if `find` is empty.
    fn replace_all(&mut self, find: &str, with: &str) -> Result<usize, EditorError>;

    /// Undoes the last step: text and cursor return to exactly what they were before it; the selection is cleared.
    /// Returns false if there is nothing to undo.
    ///
    /// Grouping: consecutive `type_text` calls (each without a selection) form ONE step, and consecutive
    /// `backspace` calls (each without a selection) form ONE step. Any other call in between (move_cursor, select,
    /// copy, cut, paste, replace_all, undo, redo, or switching between typing and backspace) ends the group.
    fn undo(&mut self) -> bool;

    /// Re-applies the last undone step (text and cursor as they were after it). False if nothing to redo.
    fn redo(&mut self) -> bool;

    fn text(&self) -> String;

    fn cursor(&self) -> usize;
}

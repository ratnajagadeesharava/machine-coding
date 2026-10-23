# 85 · Text Editor with Undo/Redo (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 60 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building the editing core of a small text editor, the part below the UI: one text buffer, a cursor, an optional selection and a clipboard. A user types, backspaces, moves the cursor, selects a range, copies/cuts/pastes and runs find-and-replace-all; the UI (not part of this problem) calls your methods and reads back `text()` and `cursor()`.

The hard part is undo/redo that feels natural. Typing a whole word is a single undo step (not one per letter), and after an undo both the text and the cursor must be exactly where they were before that step. Making a new edit after an undo throws the redo history away, as in every editor.

## Glossary

- **Cursor**: an index *between* characters, `0 <= cursor <= length`. In `"abc"`, cursor 0 is before `a`, cursor 3 is after `c`.
- **Selection**: the half-open range `[from, to)` of characters highlighted by the user; `select(from, to)` also puts the cursor at `to`. There is either one selection or none.
- **Clipboard**: one string, initially empty, written by `copy`/`cut` and read by `paste`.
- **Step**: one unit that a single `undo()` reverts and a single `redo()` re-applies. A *group* is a run of calls that together form one step.

## Requirements

1. The editor starts empty with the cursor at 0 and no selection. The cursor is an index between characters, `0 <= cursor <= length`.
2. `type(s)` replaces the selection if there is one, otherwise inserts at the cursor. Afterwards the cursor is after the inserted text and the selection is cleared. `type("")` does nothing and creates no undo step.
3. `backspace()` deletes the selection, or the character before the cursor; at position 0 without a selection it does nothing.
4. `moveCursor(pos)` clears the selection and moves the cursor. `select(from, to)` selects `[from, to)` and puts the cursor at `to`. Out-of-range arguments are errors.
5. `selectedText()` returns the selection or the empty string. `copy()` copies the selection to the clipboard (no-op without selection); `cut()` is copy plus delete (one step); `paste()` is `type(clipboard)` (one step).
6. `replaceAll(find, with)` replaces every non-overlapping occurrence scanning left to right, returns the count, and is one undo step if the count > 0. The cursor keeps its index, clamped to the new length; the selection is cleared. An empty `find` is an error.
7. `undo()` reverts the last step: text and cursor are exactly as before it, the selection is cleared. It returns false when there is nothing to undo. `redo()` re-applies the last undone step (text and cursor as after it) and returns false when there is nothing to redo.
8. Grouping: consecutive `type` calls (without a selection) are ONE step; consecutive `backspace` calls (without a selection) are ONE step. Any other call in between (moveCursor, select, copy, cut, paste, replaceAll, undo, redo, or switching between typing and backspace) ends the group.
9. A new edit clears the redo list. Operations that change neither text nor cursor-relevant state (empty type, copy, select, moveCursor) create no undo step.

## Constraints

- Single-threaded use: no locks needed (Rust: `&mut self`).
- Undo/redo must stay O(size of the change) or at worst O(text length) per step; 200 steps and 20,000 consecutive typed characters must finish quickly.
- Do not keep a full copy of the text per keystroke when a group of 20,000 characters is one step; grouping must not make the history grow with the number of calls in the group.
- Positions are character indexes (Rust: `char`s; the tests use ASCII).
- No clock: grouping is decided by the sequence of calls, never by timing.

## Examples

All traces start from a fresh, empty editor unless stated.

```
Insert at cursor
type_text("hello"); move_cursor(2); type_text("XY")   -> text "heXYllo", cursor 4

Copy / cut / paste
type_text("abcdef"); select(1, 3); copy(); move_cursor(6); paste()
                                           -> text "abcdefbc", cursor 8
select(0, 2); cut()                        -> text "cdefbc", cursor 0, clipboard "ab"
move_cursor(6); paste()                     -> text "cdefbcab", cursor 8

Typing is grouped
type_text("a"); type_text("b"); type_text("c")            -> text "abc", cursor 3 (one step)
undo()                                     -> true, text "", cursor 0
undo()                                     -> false (nothing left)
redo()                                     -> true, text "abc", cursor 3

Typing vs backspace groups
type_text("hello"); backspace(); backspace(); type_text("p")   -> text "help"
undo() -> "hel" (cursor 3)    undo() -> "hello" (cursor 5, both backspaces undone together)    undo() -> ""

Cut then paste are single steps
type_text("abcdef"); select(2, 4); cut(); paste()   -> text "abcdef", cursor 4
undo() -> "abef", cursor 2      undo() -> "abcdef", cursor 4 (where the cursor was before the cut)

replace_all
type_text("aaaa b aa"); replace_all("aa", "X")   -> returns 3, text "XX b X"
undo() -> "aaaa b aa"    redo() -> "XX b X"
replace_all("zzz", "y")                     -> returns 0, no step: the next undo() reverts the first replace_all
```

## Edge cases

- `backspace()` at cursor 0 with no selection does nothing and creates no step.
- `type_text("a")` over a selection, then `select(0, 1)` and `type_text("b")`, are two separate steps: typing over a selection never joins a group.
- `type_text("ab"); move_cursor(0); type_text("X"); type_text("Y")` is two groups: `"ab"`, then `"XY"`.
- `cut()` / `copy()` without a selection do nothing (no step, clipboard unchanged); `paste()` with an empty clipboard is `type_text("")` and does nothing.
- `move_cursor(pos)` rejects `pos < 0` or `pos > length`; `select(from, to)` rejects `from > to`, `from < 0` and `to > length`. `replace_all("", x)` is rejected.
- `replace_all` keeps the cursor index but clamps it to the new text length; the selection is cleared.
- After `undo()`/`redo()` the selection is always cleared.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `Editor` interface with `type(String)`, `backspace`, `moveCursor`, `select`, `selectedText`, `copy`, `cut`, `paste`, `replaceAll` (returns `int`), `undo`, `redo` (both `boolean`), `text`, `cursor`. Out-of-range positions and an empty `find` throw `IllegalArgumentException`.
- Rust: the trait is `Editor` with `&mut self` methods: `type_text`, `move_cursor`/`select` return `Result<(), EditorError>`, `replace_all` returns `Result<usize, EditorError>`; the only error is `InvalidArgument`.
- `type` over a selection is never grouped with a neighbouring `type` call: `type("a")` over a selection then `select`+`type("b")` are two steps.
- Typing after a `backspace` run (or backspace after typing) starts a new group.
- `undo()` and `redo()` themselves end the group, so typing after an undo starts a new step and clears the redo list.
- `undo` of a cut restores the removed text and the cursor/selection state as before the cut; after `cut` then `paste` at the same place the text is the original and the cursor is after the pasted text.
- `replaceAll` that finds nothing returns 0 and creates no step: a following `undo` reverts the previous step.
- After undo/redo the selection is cleared; the cursor equals the one recorded before/after the step.
- Backspace with a selection deletes the selection (cursor at its start) and, as a selection operation, does not join a backspace group.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
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
```

Your wiring point (`src/lib.rs`):

```rust
/// An empty document, cursor at 0.
pub fn create() -> Option<Box<dyn Editor>> { ... }
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Time-dependent behaviour uses an injectable fake clock, never real sleeping. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Insert at cursor in the middle; backspace stops at position 0.
- Typing over a selection; backspace over a selection; range errors for `moveCursor`, `select`, empty `replaceAll`.
- Copy/cut/paste results including cursor positions.
- Typing groups into one step; a cursor move ends the group; consecutive backspaces group but typing after them is its own step; typing over a selection is not grouped.
- Undo/redo restores text and cursor; a new edit clears redo; cut and paste are single undo steps.
- `replaceAll` of overlapping patterns (`aaaa b aa` with `aa` -> 3), undo/redo of it, and a zero-count replace creating no step.
- Empty `type`, `select`, `copy`, `moveCursor` do not create undo steps; 200 undo/redo cycles.
- More edge cases (a 1,000-operation randomized comparison against a model, 20,000 typed characters in one group) are in `interviewer/`.

## Running

```bash
cargo test -p p85_text_editor_undo
STRICT=1 cargo test -p p85_text_editor_undo   # fail instead of skip while unimplemented
cargo test -p p85_text_editor_undo --features hidden   # interviewer scenarios
cargo play run 85              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`FakeClock`, `sut!`): `rust/support`.

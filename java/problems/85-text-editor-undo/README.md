# 85 · Text Editor with Undo/Redo (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 60 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are writing the editing core of a small text editor, the part below the UI. There is one buffer of text, a cursor,
an optional selection, and a clipboard. Users type, backspace, move the cursor, select a range, copy, cut and paste, and use
find-and-replace-all. Undo and redo must feel natural: typing a word is one undo step, not one per letter, and after undo the
text and the cursor must be exactly where they were. A new edit after an undo discards the redo history, as in every editor.

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

```
type("hello"); moveCursor(2); type("XY")   -> text "heXYllo", cursor 4
select(0, 2); cut()                         -> text "llo", cursor 0, clipboard "he" (one step)
type("a"); type("b"); type("c")             -> one group
undo()                                      -> text "" , cursor 0 (all three typed chars revert together)
redo()                                      -> text "abc", cursor 3

type("hello"); backspace(); backspace(); type("p")
undo() -> "hel"   undo() -> "hello" (both backspaces)   undo() -> ""
```

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

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
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
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    /**
 * Wiring point between the test contract and YOUR design.
 * Create your own classes anywhere under src/main/java and return them from here.
 */
public final class Solution {
    private Solution() {}

    /** An empty document, cursor at 0. */
    public static Editor create()
}
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

## Language notes

A `StringBuilder` plus `ArrayDeque<Step>` stacks is enough. `replaceAll` is a good test of your design: it should be one step with one before-text and one after-text, and the cursor must be clamped explicitly.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/85-text-editor-undo test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`FakeClock`, `Candidate`): `java/testkit`.

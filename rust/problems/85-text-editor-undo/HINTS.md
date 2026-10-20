# Hints — 85 Text Editor with Undo/Redo (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

When the user presses undo, what is the smallest amount of information you need to have saved at the moment the edit was made?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Each undoable action is a step that knows how to apply and how to reverse itself (or the before/after state it changed), and two stacks hold the steps. Decide in one place whether a new edit starts a new step or extends the last one.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `undo` and `redo` stacks of steps, each step recording the edited position, the removed text, the inserted text, and the cursor before and after. Applying a step is the same code for the first time and for redo, undo applies its inverse. Grouping is a rule that looks at the last step and the kind of the new operation (typing or backspace, no selection, still adjacent) and merges instead of pushing; any other operation clears the 'open group' marker; any new step clears the redo stack.

**Rust:** Store steps as `struct Step { pos: usize, removed: String, inserted: String, cursor_before: usize, cursor_after: usize }` in two `Vec`s; positions in `char`s are not byte offsets (index with `char_indices` or keep the text as `Vec<char>`).

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

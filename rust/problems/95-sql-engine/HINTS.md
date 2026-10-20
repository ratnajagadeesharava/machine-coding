# Hints — 95 Mini SQL Engine (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Before any code: what are the stages between the string and the rows, and what is each stage's input and output?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Separate parsing from execution: tokenize, parse into a small statement/expression tree, then run that tree against the tables. Conditions are a tree evaluated per row; precedence falls out of how the parser nests OR above AND above comparison. For equality joins and primary-key lookups think hash maps rather than nested loops.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Validate everything (table, columns, types, key uniqueness) against the schema before touching any data, so a failing statement has nothing to undo; for UPDATE compute the new rows first, check the key constraint on the result, then swap them in. Run each statement under one lock (or a read-write lock). GROUP BY is a pass that buckets rows by key, keeping first-seen order, then folds each bucket.

**Rust:** Model tokens, statements and conditions as enums; `Vec::sort_by` is stable. Keep the tables behind one `RwLock`/`Mutex`, build the changed rows in a local `Vec`, and assign them only after all checks pass.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

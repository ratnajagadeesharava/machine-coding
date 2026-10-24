# Follow-ups — 143 Excel Cell Values and Sum Formulas

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — More functions
Add `MIN`, `MAX`, `AVG` (ignoring text and empty cells like `SUM`; `AVG` of nothing is `#DIV/0!`), `COUNT`, and `IF(condition, then, else)` with comparison operators `< > <= >= = <>`. Functions now take arbitrary expressions as arguments. How do you add a function without touching the evaluator for the others, and what does an error in the branch of `IF` that is not taken do to the dependency list and to the counter?

### V3 — Absolute and relative references
Support `$A$1`, `A$1` and `$A1`, and a `copy(from, to)` operation that rewrites relative references the way a spreadsheet does when a formula is copied. What do `formulaOf` and `dependencies` return afterwards, and what happens to references that fall off the grid (`#REF!`)?

### V4 — Several sheets and named ranges
Cells are addressed `Sheet2!B7`, and `define("Revenue", "Sheet1!B2:B40")` creates a name usable inside `SUM(Revenue)`. Where does the name table live, what happens to formulas that use a name when it is redefined or removed, and how do cycles across sheets get detected?

### V5 — Undo / redo and batching
`undo()` and `redo()` restore the previous cell contents and every value computed from them. Also add `setAll(map)` that applies several cells as one change: dependents are recalculated once for the whole batch, readers never see a state between two of the cells. Do you store inverse operations or snapshots, and what does the evaluation counter say after an undo?

### V6 — Lazy evaluation and very large sheets
The sheet grows to a million rows. Recomputing eagerly after every `set` is too expensive for a formula that sums a column of 100,000 cells. How do you keep `get` cheap and correct: dirty marking, lazy evaluation on read, incremental aggregates for `SUM`? What does "exactly once per change" mean when evaluation is lazy?

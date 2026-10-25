# Follow-ups — 133 Spreadsheet Grid

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Undo and redo
Every mutating operation (single edits, range operations, structure operations, sorts) can be undone and redone, with a bounded history of 100 steps. What does a history entry store for `insertRow` on a 200,000-cell sheet, and for `clearRange` over the whole grid? What happens to the redo stack when a new edit arrives?

### V3 — Merged cells and freeze panes
A rectangle can be merged: only its top-left cell holds a value and style; the others are hidden. Writing into a hidden cell is an error, `copy`/`sort` treat the merge as one unit, and `insertRow` inside a merge grows it. Separately, the first N rows/columns can be frozen. Which of your existing operations must now know about merges, and where does that knowledge live?

### V4 — Conditional formatting
Rules such as "number greater than 100 -> red fill" or "text starts with 'ERR' -> bold" apply to a range and change the *displayed* style of a cell without changing its stored style. Several rules can match; the first rule wins. How do rules move when rows or columns are inserted or deleted, and how does `styleOf` stay cheap for a million-cell range rule?

### V5 — Named ranges and multiple sheets
Give a rectangle a name (`Sales`), keep it correct through structure operations (it grows, shrinks, moves or becomes invalid), and let a workbook hold several sheets that can be copied between. How do you keep a name pointing at the same logical cells when rows are inserted above it?

### V6 — Large data and persistence
Load and save a sheet as a streamed file, and open a 5-million-cell file without materialising everything. What do you keep in memory, how do you page the rest, and how do edits and structure operations interact with the unloaded part?

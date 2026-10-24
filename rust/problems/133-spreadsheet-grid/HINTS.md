# Hints — 133 Spreadsheet Grid (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Out of 700 million possible cells, how many does a real sheet use? Decide what "a cell exists" means, and what the grid stores for an address that was never touched. Then ask the same question about styles: how many different ones can there be compared with the number of cells that carry one?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Store only the used cells, addressed by (row, column) in a structure that can find "everything in this row range" and "everything from this row/column on" without scanning the rest. Make a style an immutable value that is compared by content, and let cells refer to shared instances; a cell that ends up with no value and the default style should not be stored at all. For every range operation, first validate and compute the full result (a copied buffer, a sorted list of rows), then apply it, so errors and overlaps are never a problem.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `Grid { rows: ordered map row -> (ordered map column -> Cell) }`, `Cell { value: none | number | text, style: shared Style }`, `Style` an immutable value with equality and hash (the default one is a constant). `insert_row(r)`: detach the tail of the row map from key `r` on and re-insert it under key + 1 (dropping row 1,000,001); `delete_row(r)`: the same after removing key `r`. Columns do the same inside every row that has cells at or after the column. `copy`/`move_range`: read the source cells into a buffer, optionally remove the source, then write every destination cell (an absent source cell removes the destination). Sorting: take the rows' cells as lines, sort the lines by a key (group, then value) with a stable sort, write them back. `distinct_style_count`: count the distinct non-default styles that cells hold now (a reference count per style, or a set built on demand). One lock around the whole grid is enough here.

**Rust:** `BTreeMap<u32, BTreeMap<u32, Cell>>` with `split_off` for shifting; `#[derive(Clone, PartialEq, Eq, Hash)]` on `Style`, shared via `Arc<Style>`; `sort_by` is stable; `format!("{}", x)` gives the shortest decimal without an exponent, which you can round as a digit string.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

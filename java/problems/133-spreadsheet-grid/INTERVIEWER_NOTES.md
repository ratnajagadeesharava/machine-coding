# Interviewer notes — 133 Spreadsheet Grid

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- A randomised sequence of 4,000 sets, clears, style toggles, insert/delete row and column, copy and move compared against a plain-loop model (values, bold flag, `usedRange`, counts).
- `clearRange("A1","ZZ1000000")` repeated over a 200,000-cell sheet: a walk over the address space fails it.
- Sorting 50,000 rows with numbers, text, empty keys and duplicates: stability, group order, styles travelling with their row.
- Eight threads inserting rows and columns while four threads read `usedRange`/`rowCount`: the anchor ends exactly where the arithmetic says (`FE161`), and `usedRange` is never observed half-way.
- Corners: a cell at `ZZ1000000` pushed off by `insertRow(1)` or `insertColumn("ZZ")`, brought back by a delete; a styled cell removed by `deleteColumn("zz")` must also leave `distinctStyleCount`.
- Huge and tiny numbers: `1e20`, a 18-digit negative, `1e-10/3`, `0.99999999995` (rounds up to `1`), `999.9999999999`, a PERCENT of `1e20`.
- A copy that is aliased with later edits (styles are values, not references to mutable objects).

## Likely interviewer questions
- What is a cell in your model, and what does an address nobody has touched cost?
- Why did you key by (row, column) the way you did? What does inserting a row cost, what does inserting a column cost, and which is worse?
- A style change on one cell of a 1,000,000-cell range: how many style objects exist afterwards?
- How does `distinctStyleCount` stay correct when the last cell with a style is cleared or changed back to the default?
- What does `copy` do when the source overlaps the destination in each direction? How would you prove it?
- Where do you validate, and how do you guarantee that a failed `setStyleRange` or `copy` leaves nothing behind?
- How is `-0`, `0.1 + 0.2` or `1.005` displayed, and why is rounding the binary value different from rounding the shortest decimal?
- What does the sort do with `2` and `2.0`, `a` and `A`, and empty keys, ascending and descending?

## Invariants
- No stored cell is both valueless and default-styled.
- Equal styles are interchangeable; no operation mutates a style that another cell may hold.
- Every public operation validates fully before changing anything.
- After any structure operation cell contents are exactly shifted: no cell is duplicated, lost (other than the ones that fall off the grid or are deleted) or reordered.
- `usedRange`, `rowCount` and `columnCount` are always derived from the stored cells (no stale caches).

## Common mistakes
- A dense two-dimensional array or one entry per address of the range being cleared, styled or counted.
- Rebuilding the whole map for every structure operation (works for the unit tests, burns the scale test), or shifting cells one by one in the wrong direction and overwriting.
- Copying overlapping ranges cell by cell in a fixed order, which smears the first row across the others.
- `copy` that skips empty source cells instead of overwriting the destination; `move` that clears the source after pasting (erasing the overlap).
- Rounding `double` arithmetic (`value * 100`, `Math.round`, `String.format("%.2f")`) instead of the shortest decimal: `1.005`, `0.255` and negative ties come out wrong; `Math.round(-2.5)` is `-2` in Java.
- `-0` or `-0.00` printed; exponent notation (`1.0E20`) in display; trailing zeros left in `GENERAL`.
- Style equality by reference or by the colour's original case (`RED` vs `red`), so `distinctStyleCount` is too high; counting the default style; a styled empty cell not counted in `usedRange`.
- `setText("")` treated as `clear` (style lost), or `clear` that keeps the style.
- Descending sort implemented by reversing the whole ascending result: it breaks stability, puts empties first and text before numbers.
- Comparing text with `compareTo` (upper-case sorts before lower-case) or locale-sensitive collation.
- Applying part of an invalid `StyleChange` (size valid, colour invalid) before failing.
- Bounds: `insertRow(1000000)`, a destination off the grid by one row or column, a column parsed as `ZZ` -> 702 vs `AAA`, `A01`.
- Reading the used range while a structure operation is half done (no lock on read paths).

## Over-engineering traps
- A formula parser or a dependency graph (explicitly out of scope; see problem 143).
- Command objects, an event bus or a visitor hierarchy before any undo requirement exists.
- A class per number format or per style attribute, an interface for every operation.
- A spatial index (R-tree, quadtree) for rectangles when ordered maps already answer the queries.

## Design and scaling discussion
- Patterns and structures an interviewer hopes to hear: a **sparse** representation (ordered map of rows to ordered map of columns, or a map keyed by a packed `(row, col)` key plus per-row/per-column indexes); the **flyweight** for styles (immutable value objects, interned or compared by value, so cells hold references to a small set of shared styles); building the new content in a temporary buffer and then applying it (copy/sort); value objects for addresses and ranges, validated once at the boundary; a single lock (or a read-write lock) for a grid whose operations are all short.
- Structure operations: inserting a row costs the number of *rows* at or below it with a map of row maps (sub-maps are moved by pointer); inserting a column costs the number of *cells* at or right of it. A lazy offset (row/column index indirection, order-statistic tree or rope of row ids) makes row operations O(log n) at the price of a more complex lookup; a good candidate can state the trade-off even if they do the simple version.
- Memory: a million styled cells share a handful of styles; a cell is a value plus a style reference. Interning (a global table of styles) makes equality a pointer comparison and gives `distinctStyleCount` by reference counts, at the price of a lifecycle for the table.
- Number display: the shortest round-trip decimal string is the specification; `BigDecimal(Double.toString(x))` (Java) or `format!("{}", x)` (Rust) reproduce it, and rounding is then plain decimal-digit arithmetic.
- Concurrency: a single coarse lock is correct and good enough; finer locking per row breaks atomicity of structure operations and copies. A read-write lock helps only if reads dominate. Immutable snapshots (copy-on-write rows) make reads lock-free and make undo cheap, at the cost of allocation.
- Beyond memory: paging, columnar storage by column (good for sorting and sums), run-length encoding of identical styled stretches.

## Trade-off questions to ask when the candidate is done
- Row map of column maps versus one map keyed by `(row, col)`: which operations get easier or harder (insert row, insert column, usedRange, sort, clearRange)?
- Intern styles globally or compare by value? When would the interning table leak?
- Validate-then-mutate versus rollback: which would you use for `copy` and for `setStyleRange`, and what does each cost?
- If undo was required next, what does each operation record? Which of your operations are cheap to invert?
- Would you keep `rowCount` and `usedRange` cached? What invariants would that add?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`.
They only use the same public contract.

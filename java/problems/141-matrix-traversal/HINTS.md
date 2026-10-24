# Hints — 141 Matrix Traversal Strategies (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Take the 3 x 3 spiral. If I tell you "this is the 6th cell visited", can you say which row and column it is without having walked the first five? What do you need to know about the matrix to answer?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Treat each ordering as a function from the visit number to `(row, col)`. A cursor then only needs its strategy and a counter, `kthValue` is one evaluation, and `pathSum(k)` is `k` evaluations. Row-major, column-major, snake and reverse are division and remainder. For the spiral, think in rings: how many cells lie before ring `k`, and where on the ring is the remainder? For anti-diagonals, how many cells lie on the diagonals before diagonal `s`?

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: a registry `name -> position function` (built-ins and registered strategies side by side), a table `cursor id -> (strategy, index)`, and one place that turns a position into a value: check it is inside the matrix (naming the index when it is not), then read the one cell. Cells before ring `k` of an `R x C` matrix: `R*C - (R-2k)*(C-2k)`; binary search the largest `k` whose count is `<=` the index, then walk the four sides of that ring (a ring one cell wide or high is a single side). `SPIRAL_COUNTER` is `SPIRAL` on the transposed shape with row and column swapped back. Cells on anti-diagonals before `s`: a sum of triangular numbers minus the corners that fall outside the matrix; the offset gives the row inside the diagonal, and `ZIGZAG` mirrors it on even `s`. `traverse` is the one place that needs a seen-set (a bit set of `R*C` bits) to catch repeated positions.

**Java:** keep indexes and counts in `long` (`(long) rows * cols`), cast to `int` only for the final row and column, and make the cursor's `next` atomic (a small lock per cursor, or an `AtomicLong` index) rather than locking the whole service.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

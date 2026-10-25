# 141 · Matrix Traversal Strategies (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

A grid of numbers can be read in many orders: row by row, column by column, round and round in a spiral, along diagonals. You are building a **traversal service** for one read-only matrix: ask it for the whole sequence in a named order, walk the sequence one value at a time through independent cursors, jump straight to "the sum of the first k values" or "the k-th value", and plug in orders of your own. The matrices can be enormous (20,000 x 20,000 is four hundred million cells and is never stored by the caller), so walking, summing a prefix and jumping to the k-th value must cost what you ask for, not what the matrix weighs.

## Problem

An analytics tool shows heat-maps of very large grids. The grid lives behind a read-only data source (`rows()`, `cols()`, `get(r, c)`); the tool wants to scan it in the order the user picked (row by row, spiral, diagonal ...), page through the scan with cursors, preview "the first k cells" and "the k-th cell" instantly, and let a power user register a new order. Build the service. Correctness of the orders on non-square shapes (1 x N, N x 1, 2 x 5 ...) matters as much as on squares.

## Requirements

1. **The matrix.** `rows >= 1` and `cols >= 1`. The service reads values only through the provided `MatrixSource`; it must never need the whole matrix in memory. Creating the service reads no cell. A matrix built from a plain array is validated (rows of equal length, at least one row and one column) and copied, so later changes to the original array do not show.
2. **Built-in strategies**, selected by name (case-sensitive, exactly as written). For the 3 x 3 matrix `[[1,2,3],[4,5,6],[7,8,9]]`:

   | Name | Order | Result |
   |---|---|---|
   | `ROW_MAJOR` | left to right, top to bottom | 1 2 3 4 5 6 7 8 9 |
   | `COLUMN_MAJOR` | top to bottom, left to right | 1 4 7 2 5 8 3 6 9 |
   | `SNAKE` | row 0 left to right, row 1 right to left, row 2 left to right ... | 1 2 3 6 5 4 7 8 9 |
   | `SPIRAL` | clockwise from (0,0): along the top edge, down the right edge, back along the bottom edge, up the left edge, then the same one ring further in | 1 2 3 6 9 8 7 4 5 |
   | `SPIRAL_COUNTER` | counter-clockwise from (0,0): down the left edge first, along the bottom edge, up the right edge, back along the top edge, then one ring further in. It is `SPIRAL` of the transposed matrix | 1 4 7 8 9 6 3 2 5 |
   | `DIAGONAL` | anti-diagonals in the order r+c = 0, 1, 2, ...; inside one anti-diagonal the row index increases (top-right to bottom-left) | 1 2 4 3 5 7 6 8 9 |
   | `ZIGZAG` | the same anti-diagonals (LeetCode 498): even r+c is read bottom-left to top-right (row decreasing), odd r+c top-right to bottom-left (row increasing) | 1 2 4 7 5 3 6 8 9 |
   | `BOUSTROPHEDON_COLUMNS` | column 0 top to bottom, column 1 bottom to top, column 2 top to bottom ... | 1 4 7 8 5 2 3 6 9 |
   | `REVERSE_ROW_MAJOR` | the exact reverse of `ROW_MAJOR` | 9 8 7 6 5 4 3 2 1 |

   Every strategy visits every cell exactly once, on any shape, including a single row, a single column and a single cell. A ring or an anti-diagonal that is only one cell wide is simply walked once (for example `SPIRAL` and `SPIRAL_COUNTER` of one row both read it left to right and of one column both read it top to bottom).
3. `traverse(strategy)` returns all values in that order. An unknown strategy name (including the wrong case, blank or null) is an error. `traverse` is meant for matrices that fit in memory.
4. **Cursors.** `start(strategy)` opens a cursor and returns its id. Ids are 1, 2, 3, ... in order of creation, are shared by all strategies of this service and are never reused, not even after `close`. `next` returns the next value in the strategy's order or, once all `rows*cols` values were returned, an empty result (again and again). `hasNext` tells whether a value is left; `remaining` how many (`rows*cols` can exceed 32 bits, so counts are 64-bit). `reset` puts the cursor back before the first value (also after exhaustion). `close` releases the cursor; any later call with that id, a second `close` included, and any call with an id that was never issued is an error. Cursors are independent: using one never moves another.
5. **Laziness.** `start` reads no cell. Each `next` reads at most the one cell it returns. In particular, on a 20,000 x 20,000 source, `start` followed by taking the first 10 values of any built-in strategy reads at most 100 cells in total (and does not allocate memory proportional to the matrix).
6. `pathSum(strategy, k)` is the sum of the first `k` visited values, as a 64-bit number. `k` greater than `rows*cols` counts as `rows*cols`; `k = 0` gives 0; `k < 0` is an error. It reads at most `min(k, rows*cols)` cells and does not walk anything else.
7. `kthValue(strategy, k)` is the value of the visit with 0-based index `k`. `k < 0` or `k >= rows*cols` is an error. For built-in strategies it reads at most one cell and does not walk the `k` visits before it (the hidden checks use a 20,000 x 20,000 and a 100,000 x 100,000 source, the latter with more cells than a 32-bit int can count, and give a few seconds).
8. **Custom strategies.** The provided `TraversalStrategy` maps the index of a visit (0-based) to its `(row, col)`, given the matrix dimensions. `registerStrategy(name, strategy)` makes it available by name in `traverse`, `start`, `pathSum` and `kthValue` of that service (registrations are per service instance). A blank or null name, a null strategy, a built-in name and a name that is already registered are errors; the strategy of an existing registration is never replaced. Names stay case-sensitive. Registering is safe to do while other threads use the service; if several threads register the same name, exactly one succeeds.
9. A registered strategy is called only for the visits an operation really needs: `start` never, `next` at most once, `pathSum(k)` at most once for each of the first `min(k, rows*cols)` visits, `kthValue` at most once; `traverse` for each of the `rows*cols` visits.
10. **Misbehaving strategies** are detected when they are used. A position outside the matrix (negative or beyond the last row/column) makes the operation fail with an *invalid strategy output* error (Java `IllegalStateException`) whose message (Rust: the error value) names the 0-based visit index. `traverse` additionally detects a position that was already visited by an earlier visit and fails the same way with the index of the repeated visit; when several visits are wrong, the lowest index is reported. A failing `next` does not advance the cursor (calling it again fails again at the same index) and does not affect other cursors; `reset` still works. `pathSum`/`kthValue` check only the visits they ask for, and only for being inside the matrix.
11. **Concurrency.** The service is thread-safe. Many threads may start, read, reset and close their own cursors at the same time; every thread sees the full sequence. If several threads read from the *same* cursor, every value is returned to exactly one of them (none twice, none lost).

## Constraints

- Cost follows what you ask for: `start` and `next` do not depend on `rows*cols`; `pathSum` depends on `k`; `kthValue` does not depend on `k`; per-cursor state is small and does not depend on the matrix size.
- Built-in strategies must be correct for every shape, in particular 1 x N, N x 1, N x 2 and 2 x N, and for sizes where `rows*cols` does not fit in a 32-bit int.
- No randomness, no clock: the same inputs give the same sequence every time.

## Examples

```
M = [[1,2,3],[4,5,6],[7,8,9]]

SPIRAL            1 2 3 6 9 8 7 4 5          SPIRAL_COUNTER   1 4 7 8 9 6 3 2 5
DIAGONAL          1 | 2 4 | 3 5 7 | 6 8 | 9  ZIGZAG           1 | 2 4 | 7 5 3 | 6 8 | 9
```

Non-square shapes:

```
W = [[1,2,3],        T = [[1,2],
     [4,5,6]]             [3,4],
                          [5,6]]
W  SPIRAL 1 2 3 6 5 4    SPIRAL_COUNTER 1 4 5 6 3 2    DIAGONAL 1 2 4 3 5 6    ZIGZAG 1 2 4 5 3 6
T  SPIRAL 1 2 4 6 5 3    SPIRAL_COUNTER 1 3 5 6 4 2    DIAGONAL 1 2 3 4 5 6    ZIGZAG 1 2 3 5 4 6
```

Cursors, sums and errors on `M`:

```
t = create(MatrixSource.of([[1,2,3],[4,5,6],[7,8,9]]))
c = t.start("SPIRAL")                  -> 1
t.next(c) -> Optional[1]    t.next(c) -> Optional[2]    t.remaining(c) -> 7
t.reset(c); t.remaining(c)             -> 9
t.pathSum("SPIRAL", 4)                 -> 12          (1 + 2 + 3 + 6)
t.kthValue("ZIGZAG", 3)                -> 7
t.traverse("spiral")                   -> IllegalArgumentException   (names are case-sensitive)
t.kthValue("ROW_MAJOR", 9)             -> IllegalArgumentException
t.pathSum("ROW_MAJOR", 99)             -> 45          (clamped to 9)
```

Custom strategy and its validation on `M`:

```
// the test defines a strategy: the reverse of COLUMN_MAJOR
rcm = (rows, cols, index) -> { long j = rows*cols - 1 - index; return {j % rows, j / rows}; }
t.registerStrategy("REVERSE_COLUMN_MAJOR", rcm)
t.traverse("REVERSE_COLUMN_MAJOR")                  -> [9, 6, 3, 8, 5, 2, 7, 4, 1]
t.registerStrategy("REVERSE_COLUMN_MAJOR", rcm)     -> IllegalArgumentException (duplicate)
t.registerStrategy("SPIRAL", rcm)                   -> IllegalArgumentException (built-in)

// a strategy whose visit 5 repeats the cell of visit 2
t.traverse("BROKEN")                                -> IllegalStateException, message contains "5"
```

Lazy access to a huge matrix:

```
src = a test MatrixSource: 20,000 x 20,000, value = r*20000 + c, counts get() calls
t = create(src)
c = t.start("SPIRAL");  10 x t.next(c)   -> 0, 1, 2, ... 9      src.gets <= 100
t.remaining(c)                           -> 399,999,990
t.kthValue("SPIRAL", 399_999_999)        -> value of the centre-most cell, at most 1 get()
t.pathSum("ZIGZAG", 1_000)               -> at most 1,000 get() calls
```

### Edge cases

- Names are case-sensitive: `spiral` is not `SPIRAL`, but `spiral` can be registered as a new custom name.
- A 1 x 1 matrix gives its single value under every strategy; `remaining` right after `start` is 1.
- `SPIRAL_COUNTER` of `W` goes down the left edge (1, 4), along the bottom edge (5, 6), up the right edge (3), then back along the top edge (2).
- `pathSum` adds in 64 bits: three values of `Integer.MAX_VALUE` do not overflow.
- A closed cursor id is gone for good; the next `start` gets a fresh id, never the closed one.
- Two service instances over the same source do not share registered strategies or cursors.
- A registered strategy that is wrong only at visit 7 does not break operations that never reach visit 7.

## Glossary

- **Visit**: the n-th cell a strategy reads; `index` is its 0-based number.
- **Strategy**: a named visiting order (built in, or registered by the caller).
- **Cursor**: a position in one strategy's sequence that you advance with `next`; independent of every other cursor.
- **Ring**: the outer frame of the matrix; the next ring is the frame of what remains after removing the outer rows and columns.
- **Anti-diagonal**: all cells with the same `r + c`.
- **Prefix sum / path sum**: the sum of the first k values of a sequence.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `IllegalArgumentException` for: a null/blank/unknown strategy name; registering a null/blank name, a built-in name, an already registered name or a null strategy; `k < 0` (`pathSum`, `kthValue`); `k >= rows*cols` (`kthValue`); an unknown or closed cursor id; a null source or a source with zero rows or zero columns (`MatrixSource.of` also rejects a null grid and rows of different lengths). `IllegalStateException` when a registered strategy returns an invalid position (see Requirements 10).
- Orders: exactly the table in Requirements 2, for every shape (see Examples for non-square shapes).
- Read budgets (counted by the test source): `create` and `start` read 0 cells; `next` reads at most 1; `pathSum(k)` at most `min(k, rows*cols)`; `kthValue` at most 1. For a registered strategy `position` is called as described in Requirement 9.
- Cursor ids are 1, 2, 3, ... per service, never reused.
- Cursors from different threads: 16 threads each start and drain their own cursor and each sees the complete sequence; concurrent `start` calls hand out distinct ids.
- Anything not stated here (for example a matrix with more than 2^31 cells passed to `traverse`) is yours to decide.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`. `MatrixSource` and `TraversalStrategy` are **provided and implemented by the tests**; you consume them.

```java
package contract;

public interface MatrixSource {
    int rows();
    int cols();
    int get(int r, int c);          // 0 <= r < rows(), 0 <= c < cols()

    /** Immutable COPY of a grid. IllegalArgumentException for null, no rows, an empty/null row, or ragged rows. */
    static MatrixSource of(int[][] grid) { ... }          // provided
}

public interface TraversalStrategy {
    /** The {row, col} of the index-th visit (0-based), 0 <= index < rows*cols. */
    int[] position(int rows, int cols, long index);
}

/** Thread-safe. */
public interface MatrixTraversal {
    List<Integer> traverse(String strategy);
    void registerStrategy(String name, TraversalStrategy strategy);

    int start(String strategy);                 // cursor ids: 1, 2, 3, ... never reused
    Optional<Integer> next(int cursorId);       // empty once exhausted
    boolean hasNext(int cursorId);
    long remaining(int cursorId);
    void reset(int cursorId);
    void close(int cursorId);

    long pathSum(String strategy, long k);      // sum of the first k visited values, k clamped to rows*cols
    int kthValue(String strategy, long k);      // 0-based
}
```

Your wiring point (`src/main/java/solution/Solution.java`) — **the one factory signature is**:

```java
public final class Solution {
    public static MatrixTraversal create(MatrixSource source)
}
```

Tests that start from an `int[][]` call `Solution.create(MatrixSource.of(grid))`.

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. No test sleeps or uses the clock. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- All nine orders on 3 x 3, 2 x 3 and 3 x 2, single row / column / cell, and every strategy visiting every cell exactly once on all shapes up to 6 x 6.
- Error cases: unknown, misspelled and null strategy names; invalid matrices; unknown, closed and doubly closed cursors.
- Cursor protocol: next / hasNext / remaining, exhaustion, reset, independence, ids never reused.
- `pathSum` (clamping, zero, negative k, 64-bit sums) and `kthValue` (valid and invalid k).
- Custom strategies: registered and used through every operation, duplicate and built-in names rejected, per-instance registries, out-of-range and repeated positions reported with their index, cursor not advancing past a bad visit.
- Laziness: 20,000 x 20,000 matrix, `start` plus 10 values reads at most 100 cells, for all nine strategies.
- Concurrency: 16 threads each draining their own cursor; 1,600 cursors started concurrently get distinct ids.
- More (kth/prefix sums on huge matrices with read budgets and time limits, results compared against plain simulations on every shape up to 9 x 9, a shared cursor drained by 8 threads, concurrent registration) are in `interviewer/`.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/141-matrix-traversal test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`Candidate`, `Concurrently`): `java/testkit`.

# Interviewer notes — 141 Matrix Traversal Strategies

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Patterns and structures the interviewer hopes to hear
- **Strategy**: a named visiting order behind one small interface; built-ins and the test-registered strategy are treated alike, with a registry keyed by name (case-sensitive, rejecting duplicates and built-in names).
- **Iterator / cursor**: a per-cursor object holding a strategy and a single index (plus the id table for `start`/`close`), not a copy of the sequence. `reset` is "index = 0".
- **Lazy index maths**: each strategy as a pure function `index -> (row, col)`. Row/column-major, snake and reverse are div/mod; spiral is "find the ring, then the offset on the ring"; anti-diagonals are "find the diagonal by its cumulative cell count, then the offset". With that, `kthValue` is one call, `pathSum(k)` is k calls, and the cursor is a counter.
- Possibly **template method / decorator** for the shared "check position, then read the cell" step, a **registry/factory** for name lookup, and the idea that the matrix itself is a port (the `MatrixSource`) rather than a copied array.

## Hidden edge cases (see `interviewer/`)
- Every strategy against plain simulations on every shape from 1 x 1 to 9 x 9: wide, tall, single row and column, odd and even sides (the places where a spiral loop's "if top <= bottom" guards are forgotten).
- `SPIRAL_COUNTER` is the transpose of `SPIRAL`, not its reverse: the first move is down.
- `ZIGZAG` versus `DIAGONAL`: only the even anti-diagonals are flipped; the first anti-diagonal is a single cell.
- 100,000 x 100,000 = 10^10 cells: `int` index maths (`row * cols + col`, `rows * cols`, `index / cols`) overflows; `remaining` and `kthValue` must use 64-bit.
- 1 x 1,000,000 and 1,000,000 x 1: ring and diagonal counting with a degenerate dimension; an O(rows+cols) per-call helper is acceptable, an O(rows*cols) one is not.
- `kthValue` on 20,000 x 20,000 near the end of `SPIRAL`: a loop that "advances k times" passes the read-budget check and fails the time limit.
- A strategy that is only wrong at the last index (almost a permutation) fails `traverse` with exactly that index; the same strategy is fine for `pathSum(n-1)`.
- A cursor that hits a bad visit stays at that index (calling `next` again fails again), other cursors keep working, `reset` makes it usable again.
- One cursor shared by 8 threads returns every value exactly once; 16 threads race to register the same name and exactly one wins.
- 100,000 cursors opened and closed: ids are 1..100,000, the next one is 100,001, and the closed ones stay closed.
- Counting calls: `start` must not call the strategy or read a cell; `next` at most one position and one read; `kthValue` one of each.

## Likely interviewer questions
- Where does the state of a cursor live, and what is the memory cost of 100,000 open cursors on a 20,000 x 20,000 matrix?
- How did you find the ring (or the anti-diagonal) of an index without a loop over all rings? What is the complexity of `position` for your spiral?
- Why is `SPIRAL_COUNTER` not just `reverse(SPIRAL)`? Does the transposition trick also work for `BOUSTROPHEDON_COLUMNS`?
- What would you do about a registered strategy that is expensive or stateful? Who is responsible for calling it from several threads?
- Why does `traverse` check for repeats but `next` does not? What would it cost to check in a cursor?
- `next` on a closed cursor from another thread: what is the worst thing that can happen in your design?

## Invariants
- A cursor's position is in `0..=rows*cols`; `remaining = rows*cols - position`.
- Every strategy is a bijection from `0..rows*cols` onto the cells (built-ins by construction, registered ones checked on use).
- Cursor ids are issued once, in increasing order, and never reissued.
- Nothing proportional to the matrix size is allocated unless the caller asked for the whole traversal.

## Common mistakes
- Materialising the whole traversal (or the matrix) in `start`, then serving `next` from the list: correct on 3 x 3, fails the read-budget and memory checks.
- Computing `kthValue` by running a cursor `k` times.
- `int` overflow in `rows * cols`, in `remaining`, or in an index.
- Spiral loops that forget the `top <= bottom` / `left <= right` guards and emit a row or column twice on 1 x N, N x 1 and odd-by-even shapes.
- `ZIGZAG` flipping the wrong parity, or `DIAGONAL` reading each anti-diagonal bottom-up.
- Validating a registered strategy only at registration time (impossible without the dimensions) or by running the whole traversal in `start`.
- Reusing a closed cursor's id, or a `close` that does not make later calls fail.
- Non-atomic `next`: two threads sharing a cursor read the same index, or skip one.
- Throwing for an exhausted cursor instead of returning an empty result.

## Over-engineering traps
- A class hierarchy with one subclass per built-in plus a visitor, before a function from index to position would do.
- An event bus or observer for cursor lifecycle; a database-like cursor manager with leases and timeouts.
- Caching positions in a hash map "for speed".
- Generic N-dimensional machinery before V6 asks for it.

## Design and scaling discussion
- Position-as-a-function lets you do random access, sharding (V5 chunks are index ranges) and bidirectional cursors (V3) without changing the strategies; a stateful generator cannot.
- The cost of `position` per call (O(1) for the linear orders, O(log min(rows, cols)) for the spiral with a binary search over rings, O(log(rows + cols)) for the diagonals) versus an incremental stepper per cursor (O(1) amortised, but no random access): when would you keep both?
- Reading through a source interface instead of a copied array: bulk or block reads for throughput (read a row at a time for ROW_MAJOR, prefetch tiles for SPIRAL) without changing the contract.
- Where a skip predicate (V2) breaks `remaining` and `kthValue`: they become O(n) unless the predicate is structural.

## Trade-off questions to ask when the candidate is done
- Stateless `position(index)` versus a per-cursor incremental stepper: what do you gain and lose for each of the nine orders?
- Should `registerStrategy` accept a `rows x cols`-independent description instead (for example a list of moves)? What would break?
- Would you check repeated positions in `next` using a bit set per cursor? What is the memory for a 20,000 x 20,000 matrix, and is that acceptable?
- What is the contract for a source whose `get` is slow or can fail? Where would retries or caching go?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.

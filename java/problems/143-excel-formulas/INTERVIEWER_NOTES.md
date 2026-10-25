# Interviewer notes — 143 Excel Cell Values and Sum Formulas

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Patterns and structures an interviewer hopes to hear
- A **dependency graph** stored in both directions (reads / readers), maintained incrementally when a formula is replaced.
- **Topological order** of the affected sub-graph (Kahn's algorithm with in-degrees restricted to the affected set, or a DFS post-order that is iterative) so every cell is evaluated once after its inputs.
- **Cycle detection** with strongly connected components (Tarjan or Kosaraju, iterative) or Kahn leftovers plus a second pass; the subtle part is telling cells ON a cycle from cells merely downstream of one.
- A **tokenizer + recursive-descent parser** producing an **AST**, evaluated by an **interpreter** (visitor or a match over node types); the AST is also where `dependencies` comes from. Precedence climbing or one function per level are both fine.
- Value as a small **sum type** (empty / number / text / error) so error propagation is data flow, not exceptions.
- Push-based eager recomputation (this problem) versus pull-based lazy evaluation with dirty flags (V6).
- A single lock (or a read-write lock) around `set`/`get` is enough; a good answer explains why finer locking is not worth it (a `set` touches an unpredictable set of cells).

## Hidden edge cases (see `interviewer/`)
- A chain built **backwards** (last cell first): the final `set` of the head must evaluate all 999/2,599 dependents exactly once; building it is quadratic by the rules and that is accepted.
- A ring through all 2,600 cells: every cell `#CYCLE!`, no stack overflow, breaking the ring by setting one cell evaluates the other 2,599 once each (the broken cell is a number, not a formula).
- A lattice where every cell reads the whole previous row (26 x 7 cells): per-path evaluation explodes (26^7); once-per-cell evaluation needs exactly 25 + 7 x 26 evaluations.
- 26 formulas each reading the same 2,574-cell range: dependents of a single cell has 26 entries, dependencies of one formula has 2,574; 100 updates stay fast.
- A syntax-error formula counts as one evaluation; re-setting identical input re-evaluates dependents; the cells hanging off a cycle are counted once each while the cycle members are not.
- Text, number and empty transitions through bare references and `SUM`; 200-deep parentheses; ten additions of `0.1` that must display `1`.
- Readers racing writers: `D1 = C1 - B1 - B1` with `B1 = A1`, `C1 = A1*2` must always read `0`.

## Likely interviewer questions
- What exactly happens on `set(A1, ...)`: which cells are looked at, in which order, and how many are evaluated?
- Why is the dependent list stored as well as the dependency list? What does a change cost with only one of them?
- How do you tell a cell on a cycle from one that depends on a cycle? What does each one show, and what does the counter say?
- What is the depth of your recursion on the 2,600-cell chain? On the ring? On a formula with 200 nested parentheses?
- Why is a bare reference to a text cell allowed to copy the text, while `A1+0` is `#VALUE!`?
- How do you make `SUM(A1:Z99)` cheap in the graph (2,574 edges per formula, 26 such formulas)?

## Invariants
- After `set` returns, every cell's stored value equals what evaluating its formula against the stored values of the cells it reads would give (or `#CYCLE!` when on a cycle).
- `reads` and `readers` are exact mirrors; a replaced or cleared formula leaves no edge behind.
- A cell is evaluated at most once per `set`, and only after all affected cells it reads.
- The counter changes only inside evaluation.
- A failed call (invalid address, null input) changes nothing.

## Common mistakes
- Recursive recomputation ("recompute my readers, who recompute theirs"): evaluates a diamond twice (or exponentially on the lattice) and overflows the stack on the chain.
- Pull-style recursive `get` with memoisation but no invalidation, or invalidation without a correct order.
- Forgetting to remove the old edges when a formula is replaced (unrelated changes then recompute, `dependents` lists stale cells).
- Treating "left over after topological sort" as "on a cycle" — cells downstream of a cycle are not on it and must still be evaluated (and show `#CYCLE!` by propagation).
- Counting cycle members as evaluations, or not counting a syntax-error formula.
- Early cut-off ("value did not change, stop") — breaks the exact counter the contract defines.
- Sorting addresses as strings (`A10` before `A2`) instead of row by row.
- Reporting the first error in column-major order, or the right error before the left one.
- Letting `A1/0` with `A1` text give `#DIV/0!` instead of `#VALUE!`.
- Rounding with `%.10f` semantic differences, printing `-0`, printing `1E+15`, or using integer overflow for large results.
- Treating `A01`, `A0`, `AA1` as valid or as a syntax error rather than `#REF!` (formulas) / invalid address (`set`/`get`).
- `SUM` that fails on text or on empty cells; `SUM` of a range written `B3:A1` that visits the wrong order.
- Parsing the formula again on every evaluation (works, but 100 updates of a 2,600 chain become needlessly heavy).

## Over-engineering traps
- A plugin registry of functions, a type hierarchy of expression visitors, or generic number types before `SUM` and four operators work.
- A reactive-streams / observer framework between cells; a thread pool to evaluate a chain.
- Arbitrary-precision decimals; the contract is 64-bit floats with a rounding rule for display.
- Persistence, formatting, styles, sheets (those are follow-ups or another problem).

## Design and scaling discussion
- Eager push (this problem) costs O(affected) per `set` and makes `get` O(1). Lazy pull makes `set` O(1) and moves the cost to `get`, and "exactly once per change" must be redefined.
- Ranges: storing every cell of a range as an edge costs O(cells) memory per formula; interval or rectangle indexes (or per-column prefix aggregates) are the answer when the grid is huge (V6).
- Incremental `SUM`: subtract the old value and add the new one instead of re-reading the range; precision and error cells make this subtle.
- Cycle detection per `set` is local to the affected sub-graph; a global check on every `set` would be O(all cells).
- Concurrency: one lock is simplest and correct; reader-writer locks help only if `get` dominates; an immutable snapshot swapped atomically gives lock-free reads at the cost of copying.

## Trade-off questions to ask when the candidate is done
- What would change if `set` must stay under 1 ms on a 1,000,000-cell sheet?
- How would you detect that a formula became invalid because a referenced cell was deleted (rows/columns are another problem)?
- Would you keep the tree or the original text as the source of truth? What does `formulaOf` return after a future "copy formula" feature?
- How would you test the counter without making tests brittle?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.

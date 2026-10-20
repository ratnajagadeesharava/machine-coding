# Interviewer notes — 95 Mini SQL Engine

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- `INSERT` of a duplicate primary key, `UPDATE SET id = existing`, and `UPDATE SET id = 7` on several rows: nothing changes.
- Keywords in lower or mixed case, trailing `;`, `<>` vs `!=`.
- `OR` mixed with `AND` without parentheses; parentheses override.
- Numeric ordering of INT (10 after 9) and ties kept in insertion order.
- `COUNT(*)` on an empty selection returns one row with `0`.
- `SELECT name, COUNT(*)` without GROUP BY, `SUM(name)`, `LIMIT -1`, empty string, unknown column in `UPDATE ... SET`.
- Quoted text with `''` and with commas inside.
- Joins of two 20,000-row tables must not be quadratic; primary-key point lookups in a 100,000-row table must not scan.

## Likely interviewer questions
- What are your stages from string to result? What does the parser produce?
- How did you get AND-over-OR precedence, and how would you add NOT?
- How do you guarantee a failed UPDATE changes nothing?
- How does your join work, and what is its complexity? Which side did you build the hash on and why?
- How do you order text vs integers without branching on type everywhere?
- Which lock protects what? Can two SELECTs run in parallel? Can an INSERT run during a SELECT?

## Invariants
- A primary-key column never contains two equal values.
- Every row of a table has exactly one value per column, of the column's type.
- A statement is applied entirely or not at all.
- No statement observes another statement half-applied.

## Common mistakes
- Evaluating `a OR b AND c` left to right.
- Comparing integers as strings (`'10' < '9'`).
- Unstable sort or a hash-order GROUP BY that loses insertion order.
- Applying row updates in place and failing on a later row.
- Splitting VALUES on commas, which breaks `'a,b'` and `'it''s'`.
- Nested-loop join over full tables; full scan for every primary-key lookup.
- Reading a table without the lock while another thread appends to it.

## Over-engineering traps
- A general expression engine (arithmetic in SELECT/WHERE, functions) the grammar does not need.
- A query optimiser and plan classes before the straightforward executor is correct.
- Pluggable storage engines and visitor hierarchies for six statement kinds.

## Design and scaling discussion
- Real engines separate parser, planner (logical then physical plan), and executor (iterator/volcano or vectorised); even a tiny engine benefits from the first and last split.
- Storage: row store with a B-tree (SQLite, InnoDB) vs column store (analytics). The hash PK index here is a stand-in for a clustered index.
- Concurrency: one global lock (SQLite's model) is correct and simple; MVCC (PostgreSQL) lets readers never block writers; row-level locks add deadlock handling.
- Joins: nested loop, hash join, sort-merge; the choice depends on sizes, sort order and indexes.
- Atomicity without a log: validate first, then mutate; with durability you need a WAL and recovery.

## Trade-off questions to ask when the candidate is done
- One global mutex vs a read-write lock vs MVCC: what does each cost and buy?
- Hash join vs index nested-loop: when does each win?
- Storing values as strings vs typed cells: speed, memory, correctness of comparison.
- Validate-then-mutate vs undo log for atomic statements.
- How would you fuzz the parser?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.

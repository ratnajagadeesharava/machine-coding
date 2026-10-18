# Interviewer notes — 08 Tic-Tac-Toe

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- A move that completes a line in the *middle* (fills a gap) wins.
- Longer than `K` runs also win.
- Win on the last empty cell reports `WIN`, not `DRAW`.
- Anti-diagonal wins and wins near the board edge.
- Refused moves keep the turn.

## Likely interviewer questions
- How do you detect a win in O(K) per move (count along 4 axes from the last move) vs O(N) row/column counters vs O(N²) scans?
- Is the board a 2D array, a map of occupied cells, or a flat array? Which scales to 1000×1000?
- What are `Player`, `Board`, `Game` — are they all necessary? What is "state" here?
- Where should input validation live?

## Invariants
- Marks only ever get placed in empty cells, alternating X, O.
- After a win/draw, the board and turn never change.
- A refused move does not change the turn.

## Common mistakes
- Win detection that only checks the main diagonal (or only checks rows/cols from corner).
- Counting a broken line as a win (non-consecutive marks).
- Switching turns on a refused move.
- Declaring a draw before checking for a win on the final move.

## Over-engineering traps
- `Player` hierarchy with `HumanPlayer`/`BotPlayer` in V1; `WinStrategy` per direction; `Cell` objects; `GameObserver`s.

## Scalability discussion
- Counter-based O(1) detection for K = N; line-scan from the last move for general K; sparse boards for huge N.

## Trade-offs worth probing
- Simplicity vs generality: how general should V1 be given the requirement says `N×N`/`K`?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.

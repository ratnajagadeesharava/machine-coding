# Interviewer notes — 32 Board Game Engine

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- Win check must work when the placed piece completes a line in the *middle* (pieces on both sides).
- Anti-diagonal wins and six-or-more in gomoku.
- A winning move on the last empty cell is a WIN, not a DRAW.
- Absurd numbers (`99999999999,0`) are INVALID_MOVE, never an exception.
- Unknown player name is NOT_YOUR_TURN; GAME_OVER outranks everything.

## Likely interviewer questions
- What do the three games share (grid, turns, "k in a row" win) and what differs (board size, how a move maps to a cell, gravity)?
- Where do you decide *which* abstraction is shared? How would you add Reversi, where a move changes many cells?
- How do you detect a win in O(k) per move instead of scanning the board?
- How is a game looked up by name, and what is the cost of adding the fourth game?
- Where does parsing the move text live, and what happens on malformed input?

## Invariants
- Pieces on the board = number of accepted moves; P1 count − P2 count ∈ {0, 1}.
- A rejected call changes neither the board nor the turn.
- After WIN or DRAW every `play` returns GAME_OVER and the board no longer changes.
- The game name never appears in the win-detection or turn logic.

## Common mistakes
- `if (game.equals("connect4")) ... else if ...` scattered through one class (the thing this exercise is about).
- Win detection only in four directions from a corner, or only checking one side of the placed piece.
- Counting a full board as a draw before checking for a win on the last move.
- Letting a rejected move flip the turn.
- Inverting connect4 rows between storage, `cellAt` and `render`.
- `Integer.parseInt` / `parse::<usize>` blowing up or accepting `" 1"`, `"+1"` or `"1,2,3"`.

## Over-engineering traps
- `Piece`, `Cell`, `Player`, `Move`, `Board`, `GameFactory`, `GameRegistry`, `RuleEngine`, `WinStrategy`, `TurnManager`, observers... A small *rules* seam (size, parse move → cell, k-in-a-row, gravity yes/no) plus one generic game loop is usually enough; the question is how small.
- Designing for checkers' captures in V1.

## Scalability discussion
- Per-move win check scans at most `2 * (k - 1) + 1` cells per direction: O(k), independent of board size.
- Draw detection by a move counter rather than scanning for empty cells.

## Trade-offs worth probing
- Data-driven rules (a config table: rows, cols, k, gravity) vs a trait/interface per game: where does the data-driven approach break (reversi flips)?
- Storing a flat array vs a 2-D grid; mapping text → cell in the game vs in the engine.

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.

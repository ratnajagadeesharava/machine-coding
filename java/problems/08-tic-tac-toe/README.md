# 08 · Tic-Tac-Toe (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Build a Tic-Tac-Toe game — but generalized: an `N×N` board where the first player to get `K` marks in a row
(horizontally, vertically or diagonally) wins. Classic Tic-Tac-Toe is `N=3, K=3`.

## Requirements

1. Two players, `X` and `O`. `X` moves first, then they alternate.
2. A move places the current player's mark in an empty cell identified by `(row, col)`, 0-based.
3. A move is refused if the game is over, it is not that player's turn, the player is unknown, the cell is outside the board, or the cell is occupied. A refused move **does not consume the turn** and changes nothing.
4. The game is won by the player who completes `K` consecutive marks in any of the four directions. The winning move reports a win.
5. If the board is full and nobody has won, the game is a draw. A move that fills the board **and** wins is a win.
6. After the game ends no further moves are accepted.
7. The state is queryable: whose turn, the winner, the contents of any cell.

## Constraints

- Boards up to `1000×1000` with small `K`. Checking for a win after a move must not scan the whole board.
- `1 ≤ K ≤ N`.

## Examples

```
3×3, K=3:   X(0,0) O(1,0) X(0,1) O(1,1) X(0,2)  → the last move returns WIN
play("X", 0,0) twice in a row → second is NOT_YOUR_TURN, and O is still to move
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `play` returns exactly one of: `OK` (accepted, game continues), `WIN` (accepted, this move won), `DRAW` (accepted, board full), `GAME_OVER`, `NOT_YOUR_TURN`, `INVALID_PLAYER`, `OUT_OF_BOUNDS`, `OCCUPIED`. Check order: `GAME_OVER`, `INVALID_PLAYER`, `NOT_YOUR_TURN`, `OUT_OF_BOUNDS`, `OCCUPIED`.
- `nextPlayer()` is `X` or `O`, and empty once the game is over.
- `winner()` is the winning player's mark, empty while in progress or after a draw.
- `cellAt(row, col)` is `X`, `O`, or empty for an empty or out-of-bounds cell.

## Test entry point (Java)

The tests drive your system through the thin, primitive-typed contract below. It is **not** your domain
model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in
`solution/Solution.java`.

```java
package contract;

import java.util.Optional;

/** Thin test entry point. Your domain model lives behind it. */
public interface TicTacToeApi {

    /** OK, WIN, DRAW, GAME_OVER, NOT_YOUR_TURN, INVALID_PLAYER, OUT_OF_BOUNDS or OCCUPIED. */
    String play(String player, int row, int col);

    /** "X" or "O"; empty once the game is over. */
    Optional<String> nextPlayer();

    Optional<String> winner();

    /** "X", "O", or empty for an empty/out-of-bounds cell. */
    Optional<String> cellAt(int row, int col);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    TicTacToeApi create(int size, int winLength)
}
```

## Language notes

Players are the strings `"X"` and `"O"`. Keep the board representation your own decision.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/08-tic-tac-toe test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

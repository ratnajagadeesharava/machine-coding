# 32 · Board Game Engine (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 120 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

**What you are building.** A small game server hosts several two-player grid games behind one entry point. A game is
chosen by name when it is created; after that, two players (`P1`, `P2`) take turns placing pieces until somebody
completes a winning line or the board is full. Today the product ships three games, tomorrow a fourth and a fifth,
so adding one must be cheap and must not mean touching the rules of the others. A UI polls the engine for the
current board, whose turn it is, and the outcome of each attempted move.

### Terms used below

- **Cell / row / col**: a board square, addressed by zero-based `row` and `col`. For `tictactoe` and `gomoku`, row 0 is the top row. For `connect4`, row 0 is the **bottom** row.
- **Line**: consecutive cells in a straight horizontal, vertical or diagonal run.
- **Gravity** (connect4): a piece dropped in a column falls to the lowest empty cell of that column.
- **Move text**: the string describing a placement; its format depends on the game.

## Requirements

1. The engine hosts these games, selected by name: `tictactoe` (3x3, three in a row), `connect4` (7 columns x 6 rows with gravity, four in a row) and `gomoku` (15x15, five in a row).
2. Two players, `P1` and `P2`; `P1` moves first and the players alternate. A rejected move does not consume the turn.
3. A move is a piece placement described by a short text whose format depends on the game.
4. A line may run horizontally, vertically or along either diagonal. The player who completes a winning line wins immediately.
5. When the board is full and nobody has won, the game is a draw.
6. Once a game is over, no further moves are accepted.
7. The current position can be inspected cell by cell and as a text picture.
8. An unknown game name does not produce a game.

## Constraints

- Single-threaded access. One game instance per `create`; two instances never share state.

## Examples

```
create("tictactoe")   play("P1","1,1") → OK    play("P1","0,0") → NOT_YOUR_TURN    play("P2","1,1") → INVALID_MOVE
create("connect4")    play("P1","3") → OK  (the piece falls to row 0 of column 3)    play("P2","3") → OK  (row 1)
create("chess")       → no game
```

Tic-tac-toe win (moves alternate P1, P2, P1, ...; the last one returns `WIN`, earlier ones `OK`):

```
0,0  1,0  0,1  1,1  0,2     → P1 completes the top row → WIN
then play("P2","2,2")        → GAME_OVER      nextPlayer → empty      cellAt(2,2) → "."
```

Rendering (tictactoe after P1 `0,0`, P2 `1,1`, P1 `2,1`) — `render()` is `"X..\n.O.\n.X."`:

```
X..
.O.
.X.
```

Connect4 after P1 `3`, P2 `3`, P1 `0` — `cellAt(0,3)="X"`, `cellAt(1,3)="O"`, `cellAt(0,0)="X"`, and
`render()` lists the top row first, so the bottom row (row 0) is **last**:

```
.......
.......
.......
.......
...O...
X..X...
```

Gomoku: four in a row (`7,3 7,4 7,5 7,6` for P1) is still `OK`; the fifth (`7,7`) returns `WIN`. Six or more in a row also wins.

## Edge cases

- Checks run in a fixed order, first failing wins: game already over → `GAME_OVER` (even for garbage move text or the wrong player); wrong player or any player text other than `P1`/`P2` → `NOT_YOUR_TURN`; bad move → `INVALID_MOVE`.
- Invalid `tictactoe`/`gomoku` move texts: `3,0` (off a 3x3 board), `-1,0`, `x`, empty string, `1, 2` (space), `1,2,0` (three parts). Invalid `connect4` texts: `7`, `-1`, `x`, empty, `3,3`, ` 3`. An occupied cell and a full connect4 column are invalid too.
- A move that fills the last cell **and** completes a line is a `WIN`, not a `DRAW`.
- `cellAt` with coordinates off the board returns `"."`, never an error.
- Names are case-sensitive: `TicTacToe` and the empty string produce no game.

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `create(game)` is case-sensitive: only the exact names `tictactoe`, `connect4` and `gomoku` produce a game; anything else produces nothing (empty).
- Players are exactly `P1` and `P2`. `play(player, move)` returns exactly one of: `OK`, `WIN`, `DRAW`, `INVALID_MOVE`, `NOT_YOUR_TURN`, `GAME_OVER`.
- Checks are made in this order, and the first failing one decides the result: (1) the game is already over → `GAME_OVER` (whoever plays, whatever the move); (2) `player` is not the one to move (including any text other than `P1`/`P2`) → `NOT_YOUR_TURN`; (3) the move is not valid for the game → `INVALID_MOVE`. A refused call changes nothing and does not consume the turn.
- `WIN` is returned to the move that completes the line (even if it also fills the board); `DRAW` to the move that fills the board without a winner; otherwise `OK`.
- Move formats, with no spaces or other characters: `tictactoe` and `gomoku` use `row,col` (two non-negative decimal integers, row 0 is the **top** row, col 0 the left column, for example `1,2`). `connect4` uses just the column number `0`..`6` (for example `3`). Anything that does not match the format, or lies outside the board, is invalid; so is an occupied cell, or a full connect4 column.
- Connect4 uses gravity: a piece placed in a column occupies the lowest empty row of that column. Its coordinates for `cellAt` are `row 0 = bottom row`, rows `0`..`5`, columns `0`..`6`.
- Gomoku: five **or more** consecutive pieces of the same player in a line win.
- `cellAt(row, col)` is `"X"` for a `P1` piece, `"O"` for a `P2` piece and `"."` for an empty cell or any coordinates outside the board.
- `render()` is one text line per board row, joined with `\n` (no trailing newline), one character per cell (`X`, `O`, `.`), no spaces. Lines run from the **visually top row to the bottom row**: for `tictactoe` and `gomoku` that is row 0 first; for `connect4` it is row 5 first and row 0 (where pieces land) last.
- `nextPlayer()` is `P1` or `P2`, and empty once the game is over.

## Test entry point (Java)

The tests drive your system through the thin, primitive-typed contract below. It is **not** your domain
model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in
`solution/Solution.java`.

```java
package contract;

import java.util.Optional;

/** Thin test entry point. Your domain model lives behind it. One instance is one game in progress. */
public interface BoardGameApi {

    /** Returns OK, WIN, DRAW, INVALID_MOVE, NOT_YOUR_TURN or GAME_OVER. */
    String play(String player, String move);

    /** "X" (P1), "O" (P2) or "." (empty or out of the board). Connect4: row 0 is the bottom row. */
    String cellAt(int row, int col);

    /** One line per row from the visual top to the bottom, joined with "\n". */
    String render();

    /** "P1" or "P2"; empty once the game is over. */
    Optional<String> nextPlayer();
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    Optional<BoardGameApi> create(String game)
}
```

## Language notes

`create` takes the game name and returns the game, or empty for an unknown name. Everything else is on the returned object.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/32-board-game-engine test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

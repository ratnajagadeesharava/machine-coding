# 32 · Board Game Engine (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 120 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Build one engine that can host several two-player grid games. A game is chosen by name when it is created; after
that, two players take turns placing pieces until somebody wins or the board is full. Today the product ships three
games, tomorrow a fourth and a fifth, so adding one must be cheap and must not mean touching the rules of the
others.

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

- Single-threaded access. One game instance per `create`.
- Checking for a win after a move must not rescan the whole board (gomoku is 225 cells).
- Adding another grid game should not require editing the code paths of the existing ones.

## Examples

```
create("tictactoe")   play("P1","1,1") → OK    play("P1","0,0") → NOT_YOUR_TURN    play("P2","1,1") → INVALID_MOVE
create("connect4")    play("P1","3") → OK  (the piece falls to row 0 of column 3)    play("P2","3") → OK  (row 1)
create("chess")       → no game
```

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

## Test entry point (Rust)

The tests drive your system through the thin, primitive-typed contract below (`src/contract.rs`). It is
**not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up
in `src/lib.rs`.

```rust
//! Thin test entry point. Your domain model lives behind it.
//! One instance is one game in progress.

pub trait BoardGameApi {
    /// Returns OK, WIN, DRAW, INVALID_MOVE, NOT_YOUR_TURN or GAME_OVER.
    fn play(&mut self, player: &str, mv: &str) -> String;

    /// "X" (P1), "O" (P2) or "." (empty or out of the board). Connect4: row 0 is the bottom row.
    fn cell_at(&self, row: i32, col: i32) -> String;

    /// One line per row from the visual top to the bottom, joined with "\n".
    fn render(&self) -> String;

    /// "P1" or "P2"; `None` once the game is over.
    fn next_player(&self) -> Option<String>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(_game: &str) -> Option<Box<dyn BoardGameApi>> {
    None
}
```

## Language notes

`create` takes the game name and returns `Option<Box<dyn BoardGameApi>>` (`None` for unknown names).

## Running

```bash
cargo test -p p32_board_game_engine
STRICT=1 cargo test -p p32_board_game_engine   # fail instead of skip while unimplemented
cargo test -p p32_board_game_engine --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p32_board_game_engine --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

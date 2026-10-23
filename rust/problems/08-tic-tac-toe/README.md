# 08 · Tic-Tac-Toe (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building the rules engine for a Tic-Tac-Toe game that has been generalised: the board is `N×N`, and the first
player to put `K` of their marks in an unbroken straight line wins. Classic Tic-Tac-Toe is `N=3, K=3`; `N=5, K=4`
is "four in a row on a 5×5 board". Two players, `X` and `O`, send moves to the engine one at a time; the engine says
whether each move was accepted, whether it ended the game, and can be asked at any moment whose turn it is, who won,
and what is in any cell. There is no UI and no AI opponent.

### Terms

- **Mark** - the symbol a player places: `"X"` or `"O"`. A player is identified by their mark.
- **Cell** - one square, addressed `(row, col)`, both 0-based, `(0,0)` top-left; `0 ≤ row, col < N`.
- **Line of K** - `K` cells in a row that are adjacent and all hold the same mark, going horizontally, vertically,
  diagonally (down-right) or anti-diagonally (down-left). A longer run than `K` also counts. Runs do not wrap around edges.
- **Refused move** - a move that is rejected with an error-style result (see the table below).

## Requirements

1. Two players, `X` and `O`. `X` moves first, then they alternate strictly.
2. A move places the given player's mark in an empty cell identified by `(row, col)`, 0-based.
3. A move is refused if the game is over, it is not that player's turn, the player is unknown, the cell is outside the board, or the cell is occupied. A refused move **does not consume the turn** and changes nothing (the same player is still to move, the board is untouched).
4. The game is won by the player who completes `K` consecutive marks in any of the four directions. The winning move reports a win. A run longer than `K` also counts.
5. If the board is full and nobody has won, the game is a draw. A move that fills the board **and** wins is a win, not a draw.
6. After the game ends (win or draw) no further moves are accepted; every attempt returns `GAME_OVER` and changes nothing.
7. The state is queryable at any time: whose turn it is, the winner, the contents of any cell.

### Results of a move

| Result | Meaning | Turn consumed? |
|---|---|---|
| `OK` | accepted, game continues | yes |
| `WIN` | accepted, this move completed a line of `K` | game over |
| `DRAW` | accepted, board now full, no winner | game over |
| `GAME_OVER` | the game had already ended | no |
| `INVALID_PLAYER` | player is not exactly `"X"` or `"O"` | no |
| `NOT_YOUR_TURN` | it is the other player's turn | no |
| `OUT_OF_BOUNDS` | `row` or `col` is negative or `>= N` | no |
| `OCCUPIED` | the cell already holds a mark | no |

If several reasons apply at once, the **first** one in this order is reported: `GAME_OVER`, `INVALID_PLAYER`,
`NOT_YOUR_TURN`, `OUT_OF_BOUNDS`, `OCCUPIED`.

## Constraints

- Boards up to `1000×1000` with small `K`. Checking for a win after a move must not scan the whole board.
- `1 ≤ K ≤ N`.

## Examples

Classic 3×3, `K=3`. Moves are `player(row,col)`:

```
X(0,0) -> OK            board: X . .     next player: O
O(1,0) -> OK                   O . .
X(0,1) -> OK                   . . .
O(1,1) -> OK
X(0,2) -> WIN           top row is X X X; winner() = X; next_player() = empty
O(2,2) -> GAME_OVER     cell (2,2) stays empty
```

Refused moves do not cost the turn:

```
O(0,0)  -> NOT_YOUR_TURN   (X moves first; board unchanged, X still to move)
X(0,0)  -> OK
X(1,1)  -> NOT_YOUR_TURN   (O is to move)
O(0,0)  -> OCCUPIED        (O is still to move)
O(3,0)  -> OUT_OF_BOUNDS
Z(0,1)  -> INVALID_PLAYER
O(0,1)  -> OK              (the turn was never lost)
```

Shorter line on a bigger board is not a win (`N=5, K=4`): X holds `(0,0) (0,1) (0,2)` in a row and O is placing
elsewhere -> every move is `OK` and `winner()` is empty. X wins only when a fourth consecutive mark lands, e.g.
the diagonal `(0,0) (1,1) (2,2) (3,3)` returns `WIN` on its last move.

Draw (3×3): `X(0,0) O(0,1) X(0,2) O(1,1) X(1,0) O(1,2) X(2,1) O(2,0) X(2,2)` ->
the ninth move returns `DRAW`; `winner()` is empty; `next_player()` is empty.

## Edge cases

- `K = 1`: the very first accepted move wins. `K = N`: only full rows, full columns or the two main diagonals win.
- A negative coordinate (`-1`) is out of bounds, not an error/crash.
- A refused move never changes the board, the turn or the winner.
- Querying a cell outside the board returns empty (it is not an error).

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `play` returns exactly one of: `OK` (accepted, game continues), `WIN` (accepted, this move won), `DRAW` (accepted, board full), `GAME_OVER`, `NOT_YOUR_TURN`, `INVALID_PLAYER`, `OUT_OF_BOUNDS`, `OCCUPIED`. Check order: `GAME_OVER`, `INVALID_PLAYER`, `NOT_YOUR_TURN`, `OUT_OF_BOUNDS`, `OCCUPIED`.
- `nextPlayer()` is `X` or `O`, and empty once the game is over.
- `winner()` is the winning player's mark, empty while in progress or after a draw.
- `cell_at(row, col)` is `X`, `O`, or empty for an empty or out-of-bounds cell.

## Test entry point (Rust)

The tests drive your system through the thin, primitive-typed contract below (`src/contract.rs`). It is
**not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up
in `src/lib.rs`.

```rust
//! Thin test entry point. Your domain model lives behind it.

pub trait TicTacToeApi {
    /// OK, WIN, DRAW, GAME_OVER, NOT_YOUR_TURN, INVALID_PLAYER, OUT_OF_BOUNDS or OCCUPIED.
    fn play(&mut self, player: &str, row: i32, col: i32) -> String;

    /// "X" or "O"; `None` once the game is over.
    fn next_player(&self) -> Option<String>;

    fn winner(&self) -> Option<String>;

    /// "X", "O", or `None` for an empty/out-of-bounds cell.
    fn cell_at(&self, row: i32, col: i32) -> Option<String>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(_size: u32, _win_length: u32) -> Option<Box<dyn TicTacToeApi>> {
    None
}
```

## Language notes

The contract methods take `&mut self` for `play` and `&self` for queries. Players and results are the plain strings listed above; coordinates are `i32` so negative values can be passed in and must be reported as `OUT_OF_BOUNDS`.

## Running

```bash
cargo test -p p08_tic_tac_toe
STRICT=1 cargo test -p p08_tic_tac_toe   # fail instead of skip while unimplemented
cargo test -p p08_tic_tac_toe --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p08_tic_tac_toe --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

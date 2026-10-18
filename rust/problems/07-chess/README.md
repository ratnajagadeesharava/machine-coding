# 07 · Chess (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Senior | 120 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Implement a chess game engine: it knows the position, whose turn it is, which moves are legal, applies moves, and
reports the state of the game (in progress, check, checkmate, stalemate).

Chess is not an exercise in inheriting `King extends Piece`. It is an exercise in deciding what the *concepts* are,
who owns the rules, and how to keep a large rule set correct. You decide the model.

## Requirements

1. Standard 8×8 chess. White moves first; players alternate.
2. All piece movement rules: sliding pieces cannot jump; knights jump; pawns move forward one (two from their start rank), capture diagonally, and never capture forward.
3. **Captures** remove the captured piece from the board.
4. A move is **legal** only if it does not leave the mover's own king in check. A king cannot move into check, pinned pieces cannot expose their king.
5. **Check** is reported when the side to move is attacked. **Checkmate**: in check with no legal move. **Stalemate**: not in check with no legal move.
6. **Castling** (both sides): king and rook unmoved, squares between empty, king not in check, does not pass through or land on an attacked square.
7. **En passant**: only available immediately after the opponent's two-square pawn advance.
8. **Promotion**: a pawn reaching the last rank must promote to queen, rook, bishop or knight (the move names the piece).
9. After checkmate or stalemate no further moves are accepted.
10. A game can start from the standard position or from a position described in **FEN** (used by tests to set up scenarios).

## Constraints

- The engine must be deterministic and fast enough to compute legal moves for thousands of positions (tests enumerate tens of thousands of positions by replaying moves).
- Invalid input (bad notation, empty square, wrong colour, illegal move) is a refusal, never a crash.
- Draws by repetition, the fifty-move rule and insufficient material are **not** required in V1.

## Examples

```
new game: legalMoves() has 20 moves.
move("e2e4") → OK     turn() = BLACK     pieceAt("e4") = "wP"
move("e2e5") → ILLEGAL (not a legal pawn move)

Fool's mate: f2f3 e7e5 g2g4 d8h4 → status() = CHECKMATE
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- **Squares** are `a1`…`h8` (file letter, rank digit). **Moves** use coordinate notation: `e2e4`; promotion appends the piece letter `q`, `r`, `b` or `n`: `e7e8q`. Castling is the king's two-square move (`e1g1`, `e1c1`, `e8g8`, `e8c8`).
- `pieceAt(square)` returns `wP wN wB wR wQ wK` / `bP … bK`, or empty for an empty square (or an invalid square).
- `turn()` is `WHITE` or `BLACK`. `status()` is `IN_PROGRESS`, `CHECK`, `CHECKMATE` or `STALEMATE` (from the point of view of the side to move).
- `move(m)` returns `OK` (applied), `ILLEGAL` (anything that is not a legal move, including malformed input and a promotion move without a piece letter or a non-promotion move with one), or `GAME_OVER` (checkmate/stalemate already reached).
- `legalMoves()` lists every legal move for the side to move in notation above, sorted alphabetically; empty when the game is over. For promotions each of the four pieces is a separate move.
- `fromFen(fen)` supports piece placement, side to move, castling rights and the en-passant target square; the halfmove/fullmove counters may be present and are ignored. It returns an engine in that position.

## Test entry point (Rust)

The tests drive your system through the thin, primitive-typed contract below (`src/contract.rs`). It is
**not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up
in `src/lib.rs`.

```rust
//! Thin test entry point. Your domain model lives behind it.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MoveResult {
    Ok,
    Illegal,
    GameOver,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    InProgress,
    Check,
    Checkmate,
    Stalemate,
}

pub trait ChessApi {
    /// Applies a move in coordinate notation ("e2e4", "e7e8q").
    fn make_move(&mut self, mv: &str) -> MoveResult;

    /// "WHITE" or "BLACK".
    fn turn(&self) -> String;

    fn status(&self) -> Status;

    /// "wP", "bK", ... or `None` for an empty/invalid square.
    fn piece_at(&self, square: &str) -> Option<String>;

    /// All legal moves for the side to move, sorted alphabetically.
    fn legal_moves(&self) -> Vec<String>;
}

#[allow(clippy::wrong_self_convention)]
pub trait ChessFactory {
    fn new_game(&self) -> Box<dyn ChessApi>;

    /// Creates a game from a FEN string used to set up positions.
    fn from_fen(&self, fen: &str) -> Box<dyn ChessApi>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create() -> Option<Box<dyn ChessFactory>> {
    None
}
```

## Language notes

`ChessApi` is `&mut self`. Rust makes copy-on-move cheap for a small `Copy` position type and encourages enums for piece kinds. Think about `[Option<Piece>; 64]` vs bitboards, and whether `Move` should be a plain `Copy` struct.

## Running

```bash
cargo test -p p07_chess
STRICT=1 cargo test -p p07_chess   # fail instead of skip while unimplemented
cargo test -p p07_chess --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p07_chess --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

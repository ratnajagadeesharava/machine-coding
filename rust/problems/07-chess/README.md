# 07 · Chess (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Senior | 120 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

**What you are building.** The rules engine of a chess program, with no user interface. Given a game, it knows where every piece stands, whose turn it is, and exactly which moves are legal (including the special moves: castling, en passant and promotion). A caller submits moves as text such as `e2e4`; the engine applies legal ones, refuses illegal ones without changing anything, and reports the state of the game: in progress, check, checkmate or stalemate. Games start from the standard setup or from a position given in FEN notation, which is how tests create specific situations.

## Requirements

1. Standard 8×8 chess. White moves first; players alternate. `new_game()` starts from the standard setup; `from_fen(fen)` starts from a given position.
2. All piece movement rules: sliding pieces cannot jump; knights jump; pawns move forward one (two from their start rank), capture diagonally, and never capture forward.
3. **Captures** remove the captured piece from the board.
4. A move is **legal** only if it does not leave the mover's own king in check. A king cannot move into check, pinned pieces cannot expose their king (a *pinned* piece is one standing between its own king and an enemy sliding piece that would then attack the king). While in check, only moves that remove the check are legal (move the king, capture the checker, or block).
5. **Check** is reported when the side to move is attacked. **Checkmate**: in check with no legal move. **Stalemate**: not in check with no legal move.
6. **Castling** (both sides): king and rook unmoved, squares between empty, king not in check, does not pass through or land on an attacked square.
7. **En passant**: a pawn that is beside an enemy pawn which has *just* moved two squares forward may capture it diagonally onto the square it skipped; the captured pawn is removed. Available only on the very next move, not later.
8. **Promotion**: a pawn reaching the last rank must promote to queen, rook, bishop or knight (the move names the piece).
9. After checkmate or stalemate no further moves are accepted: `make_move` returns `GAME_OVER` and `legal_moves` is an empty list.
10. A game can start from the standard position or from a position described in **FEN** (used by tests to set up scenarios).

## Constraints

- The engine must be deterministic and fast enough to compute legal moves for thousands of positions (tests enumerate tens of thousands of positions by replaying moves).
- Invalid input (bad notation, empty square, wrong colour, illegal move) is a refusal, never a crash.
- Draws by repetition, the fifty-move rule and insufficient material are **not** required in V1.

## Examples

**1. Opening moves**

```
new_game():  turn() = WHITE   status() = IN_PROGRESS   legal_moves() has 20 moves
piece_at("e1") = "wK"   piece_at("d8") = "bQ"   piece_at("e4") = `None`   piece_at("z9") = `None`
make_move("e2e4") -> OK       turn() = BLACK   piece_at("e4") = "wP"   piece_at("e2") = `None`
make_move("e2e5") -> ILLEGAL  (pawn cannot advance three; state unchanged)
make_move("e7e5") when it is White's turn -> ILLEGAL
make_move("a1a3") -> ILLEGAL  (rook blocked by its own pawn)    make_move("nonsense"), make_move("") -> ILLEGAL
```

**2. Fool's mate**

```
f2f3  e7e5  g2g4  d8h4        -> every move OK
status() = CHECKMATE   legal_moves() = []   make_move("a2a3") -> GAME_OVER
```

**3. King safety and check** (FEN `3rk3/8/8/8/8/8/8/4K3 w - - 0 1`: white king e1, black rook d8, black king e8)

```
make_move("e1d1") -> ILLEGAL, make_move("e1d2") -> ILLEGAL   (both on the rook's d-file)
legal_moves() = ["e1e2", "e1f1", "e1f2"]
```
Check: FEN `4r2k/8/8/8/8/8/8/R3K3 w - - 0 1` gives `status() = CHECK` and `legal_moves() = ["e1d1","e1d2","e1f1","e1f2"]` (the rook on e8 attacks the king; the white rook cannot help). Pin: in `4r2k/8/8/8/8/8/4B3/4K3 w - - 0 1` the bishop on e2 is pinned, so no legal move starts with `e2`.

**4. Stalemate**: FEN `7k/5Q2/6K1/8/8/8/8/8 b - - 0 1` has Black to move, not in check, with no legal move: `status() = STALEMATE`, `legal_moves() = []`, `make_move("h8g8")` -> `GAME_OVER`.

**5. Castling** (FEN `r3k2r/8/8/8/8/8/8/R3K2R w KQkq - 0 1`)

```
make_move("e1g1") -> OK   king on g1, rook from h1 to f1, h1 empty
make_move("e8c8") -> OK   king on c8, rook from a8 to d8, a8 empty
```
Refused (`ILLEGAL`) with White to move: king currently in check (`4r2k/.../R3K2R w KQ`); crossing an attacked f1 (`5r1k/.../R3K2R`); landing on attacked g1 (`6rk/.../R3K2R`), yet `e1c1` is then still `OK`; a piece between king and rook (`R2QK2R`, `e1c1`); no castling rights (`w - -`); or after the king has moved away and back (`e1e2 e8e7 e2e1 e7e8` then `e1g1`).

**6. En passant**

```
e2e4 a7a6 e4e5 d7d5      -> white pawn e5 may now capture:  make_move("e5d6") -> OK
                            piece_at("d6") = "wP", piece_at("d5") = `None` (captured pawn removed)
e2e4 a7a6 e4e5 d7d5 h2h3 a6a5 -> one move too late:  make_move("e5d6") -> ILLEGAL
```

**7. Promotion** (FEN `8/P6k/8/8/8/8/8/K7 w - - 0 1`)

```
make_move("a7a8")  -> ILLEGAL (a piece letter is required)
make_move("a7a8k") , make_move("a7a8p") -> ILLEGAL (only q, r, b, n)
legal_moves() contains "a7a8q" "a7a8r" "a7a8b" "a7a8n"
make_move("a7a8n") -> OK   piece_at("a8") = "wN", piece_at("a7") = `None`
```

## Edge cases

- An illegal or malformed move never changes the position, side to move or status.
- A promotion letter on a move that is not a promotion (e.g. `e2e4q`) is `ILLEGAL`.
- Pawns never capture straight ahead and never move diagonally unless capturing (or capturing en passant).
- Castling rights are lost when the king or the relevant rook has moved; losing one side's right does not affect the other side's.
- Move counts are a good correctness check: the starting position has 20 legal moves, 400 positions after two plies and 8902 after three (see "Move counts" below).

## Move counts for checking your engine

Counting every legal move sequence of a given length ("perft") is a standard test. The tests require these totals (White to move in each FEN):

| Position (FEN) | Legal moves now | Positions after 2 plies |
|---|---|---|
| Start position | 20 | 400 (8902 after 3) |
| `r3k2r/p1ppqpb1/bn2pnp1/3PN3/1p2P3/2N2Q1p/PPPBBPPP/R3K2R w KQkq - 0 1` | 48 | 2039 |
| `8/2p5/3p4/KP5r/1R3p1k/8/4P1P1/8 w - - 0 1` | 14 | 191 |
| `r3k2r/Pppp1ppp/1b3nbN/n_p6/BBP1P3/q4N2/Pp1P2PP/R2Q1RK1 w kq - 0 1` | 6 | 264 |
| `rnbq1k1r/pp1Pbppp/2p5/8/2B5/8/PPP1NnPP/RNBQK2R w KQ - 1 8` | 44 | 1486 |

## Glossary

- **Rank / file**: rows 1-8 and columns a-h of the board; `e4` is file e, rank 4.
- **FEN**: a one-line text description of a position: piece placement (rank 8 first, `/`-separated, uppercase = white, digits = empty squares), side to move (`w`/`b`), castling rights (`KQkq`, or `-`), en-passant target square (or `-`), then optional move counters.
- **Check / checkmate / stalemate**: king attacked / attacked with no legal move / not attacked with no legal move.
- **Pin**: a piece that cannot move off a line without exposing its own king.
- **Ply**: one move by one side.

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- **Squares** are `a1`…`h8` (file letter, rank digit). **Moves** use coordinate notation: `e2e4`; promotion appends the piece letter `q`, `r`, `b` or `n`: `e7e8q`. Castling is the king's two-square move (`e1g1`, `e1c1`, `e8g8`, `e8c8`).
- `piece_at(square)` returns `wP wN wB wR wQ wK` / `bP … bK`, or `None` for an empty square (or an invalid square).
- `turn()` is `WHITE` or `BLACK`. `status()` is `IN_PROGRESS`, `CHECK`, `CHECKMATE` or `STALEMATE` (from the point of view of the side to move).
- `make_move(m)` returns `OK` (applied), `ILLEGAL` (anything that is not a legal move, including malformed input and a promotion move without a piece letter or a non-promotion move with one), or `GAME_OVER` (checkmate/stalemate already reached).
- `legal_moves()` lists every legal move for the side to move in notation above, sorted alphabetically; an empty list when the game is over. For promotions each of the four pieces is a separate move.
- `from_fen(fen)` supports piece placement, side to move, castling rights and the en-passant target square; the halfmove/fullmove counters may be present and are ignored. It returns an engine in that position.
- Once the game is over (`CHECKMATE` or `STALEMATE`), `make_move` returns `GAME_OVER` for any input.
- `legal_moves()` for a position with promotion lists four moves per promoting pawn move/capture, e.g. `a7a8b`, `a7a8n`, `a7a8q`, `a7a8r`, in alphabetical order with all other moves.

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

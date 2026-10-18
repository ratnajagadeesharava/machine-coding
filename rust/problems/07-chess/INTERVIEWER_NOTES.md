# Interviewer notes — 07 Chess

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- Perft counts at higher depth for several well-known positions (they catch en passant, castling, promotion and pin bugs that targeted tests miss).
- En passant that would expose the king along the rank (the "horizontal pin" trap).
- Castling rights lost when a *rook is captured* on its home square.
- Promotion with capture; underpromotion.
- Check that can be answered by capture, block, or king move; double check (only king moves).
- Stalemate vs checkmate distinction with a lone king.

## Likely interviewer questions
- What are your core concepts? Is a "piece" behavior (polymorphism), data (enum + tables), or both? Why?
- Where do the *rules* live: in the pieces, in the board, in a rules module? What does a `Move` know?
- How do you generate legal moves — pseudo-legal then filter by "king not attacked", or something smarter? Cost?
- How do you apply and undo a move — copy the board or make/unmake? What state does a move change (castling rights, en-passant square, captured piece)?
- How do you test move generation? (Perft!)
- What would you change to support Chess960 / another board game?

## Invariants
- Exactly one king per side; the side that just moved is never in check.
- Castling rights only ever decrease; the en-passant target exists only for one ply.
- `legalMoves()` ⊆ pseudo-legal moves and every legal move, when applied, leaves the mover's king safe.
- Board occupancy after `move` differs from before in exactly the squares touched by that move.

## Common mistakes
- Checking castling-through-check only for the destination square.
- Forgetting to remove the en-passant pawn (it is not on the destination square).
- Not clearing the en-passant target after one ply.
- Pawn attacking vs pawn moving confusion (pawns attack diagonally even when no piece is there, for king-safety checks).
- Promotion moves treated as one move instead of four.
- Updating castling rights on king/rook move but not on rook capture.
- Mutating the live board to test legality and forgetting to restore it (or leaking the mutation on `ILLEGAL`).

## Over-engineering traps
- `Piece` hierarchy + `PieceFactory` + `MoveStrategy` per piece + `Visitor` + `GameState` pattern objects + `Command` for moves, all in V1.
- Interfaces for `Board`, `Player`, `Game`, `Rule`, `Validator` each with one implementation.
- A `Player` class that does nothing but hold a colour.
- Modelling squares as objects with piece references both ways (two sources of truth).

## Scalability / concurrency discussion
- Pure functions on immutable positions make concurrent games trivial; mutable boards need per-game ownership.
- Bitboards vs array-of-pieces; transposition keys for repetition (Zobrist).

## Trade-offs worth probing
- Polymorphic piece objects vs enum + direction tables: the first reads nicely, the second makes move generation, attack detection and testing much smaller. Neither is wrong; weigh it.
- Immutable position (copy per move) vs make/unmake.
- Pseudo-legal + filter vs legal generation.

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.

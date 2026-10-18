# 09 · Snake and Ladder (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Implement the Snake and Ladder board game. The board, the snakes and ladders and the players are configurable.
To keep the game deterministic and testable, the **dice value is supplied by the caller** — the game does not roll
dice itself.

## Requirements

1. The board has squares `1..size`. Every player starts **off the board** at position `0`.
2. Players take turns in the order they were registered. A turn is one call to `roll(dice)` with a value `1..6` for the player whose turn it is.
3. A player moves forward by the dice value. If the move would pass the last square, the player **stays** where they are (an exact roll is needed to finish) — the turn still passes.
4. Landing on the **head** of a snake moves the player down to its tail; landing on the **bottom** of a ladder moves the player up to its top. Jumps do not chain (the destination of a jump is final).
5. The first player to reach the last square — by walking or by a ladder — wins. After that no more moves are accepted.
6. Dice values outside `1..6` are refused and do not consume the turn.
7. The state is queryable: whose turn it is, each player's position, the winner.

## Constraints

- `size` is between 10 and 10,000; snake heads and ladder bottoms are distinct squares in `2..size-1`; a snake's tail is below its head; a ladder's top is above its bottom.
- 1 to 100 players.

## Examples

```
size 30, snakes {27→5, 21→9}, ladders {3→22, 8→26}, players [a, b]
roll(3) → a lands on ladder bottom 3 → moves to 22, returns 22     (b's turn)
roll(1) → b moves to 1
roll(5) → a lands on 27, a snake head → moves to 5
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `roll(dice)` returns the **new position of the player who just moved** (after any snake/ladder), or `-1` if the roll was refused (game over or invalid dice).
- `currentPlayer()` is the name of the player whose turn it is; empty after the game is over.
- `positionOf(player)` is `0` before entering the board, `-1` for unknown players.
- `winner()` is the winning player's name, empty while the game is in progress.

## Test entry point (Rust)

The tests drive your system through the thin, primitive-typed contract below (`src/contract.rs`). It is
**not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up
in `src/lib.rs`.

```rust
//! Thin test entry point. Your domain model lives behind it.

pub trait SnakeAndLadderApi {
    /// Applies the dice value for the current player. Returns the mover's new position, or -1 if refused.
    fn roll(&mut self, dice: i32) -> i32;

    /// `None` after the game is over.
    fn current_player(&self) -> Option<String>;

    /// 0 before entering the board; -1 for unknown players.
    fn position_of(&self, player: &str) -> i32;

    fn winner(&self) -> Option<String>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(
    _size: u32,
    _snakes: &[(u32, u32)],
    _ladders: &[(u32, u32)],
    _players: &[&str],
) -> Option<Box<dyn SnakeAndLadderApi>> {
    None
}
```

## Language notes

The factory receives snakes and ladders as slices of `(from, to)` pairs and players as `&[&str]`.

## Running

```bash
cargo test -p p09_snake_and_ladder
STRICT=1 cargo test -p p09_snake_and_ladder   # fail instead of skip while unimplemented
cargo test -p p09_snake_and_ladder --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p09_snake_and_ladder --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

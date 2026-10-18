# Interviewer notes — 09 Snake and Ladder

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- Overshoot passes the turn without moving.
- A ladder whose top is the last square wins immediately.
- A player whose jump lands them at a square that is the *start* of another jump does not chain.
- Invalid dice do not consume the turn; rolls after the winner are refused.
- Snakes and ladders configured on the same square are invalid (state your assumption).

## Likely interviewer questions
- Is the dice part of the game? Why did we inject it? How would you plug in a real random dice?
- How are snakes and ladders represented? One map `from → to`, or two classes? Does the game need to know which is which?
- What is `Player` — a class, or a name plus a position in a map?
- What is the turn model? An index into a list, a queue rotating? How do extra turns affect it?

## Invariants
- Positions are in `[0, size]`; only the winner is at `size`.
- Exactly one player is current while the game runs.
- A snake never moves a player up and a ladder never moves one down.

## Common mistakes
- Moving first and checking for the end afterwards (positions beyond `size`).
- Chaining jumps (landing on a ladder top that is a snake head).
- Advancing the turn on refused dice.
- Treating "reaches exactly `size`" differently for walking vs ladders.

## Over-engineering traps
- `Board`, `Cell`, `Snake`, `Ladder`, `Jumper` interface, `Dice` interface, `DiceStrategy`, `GameObserver`, `PlayerFactory` — for a 45-minute problem.
- Modelling every square as an object.

## Scalability discussion
- Jumps as a sparse map; thousands of concurrent games each owning its state.

## Trade-offs worth probing
- A single `jumps: Map<Int,Int>` merges snakes and ladders: simpler, but loses the ability to validate or render them separately. When does that matter?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.

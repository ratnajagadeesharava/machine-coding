# 09 · Snake and Ladder (Java)

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

## Test entry point (Java)

The tests drive your system through the thin, primitive-typed contract below. It is **not** your domain
model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in
`solution/Solution.java`.

```java
package contract;

import java.util.Optional;

/** Thin test entry point. Your domain model lives behind it. */
public interface SnakeAndLadderApi {

    /** Applies the dice value for the current player. Returns the mover's new position, or -1 if refused. */
    int roll(int dice);

    /** Empty after the game is over. */
    Optional<String> currentPlayer();

    /** 0 before entering the board; -1 for unknown players. */
    int positionOf(String player);

    Optional<String> winner();
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    SnakeAndLadderApi create(int size, Map<Integer, Integer> snakes, Map<Integer, Integer> ladders, List<String> players)
}
```

## Language notes

Use whatever collections you like for snakes/ladders; the factory receives both as maps.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/09-snake-and-ladder test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

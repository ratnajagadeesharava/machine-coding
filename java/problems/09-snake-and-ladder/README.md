# 09 · Snake and Ladder (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building the rules engine for the classic Snake and Ladder board game. Players race along a numbered track
from square 1 to the last square. Landing on the foot of a ladder carries a player up; landing on a snake's head
drops them down. Because real dice are random, **the dice value is supplied by the caller** on every turn - the
engine never rolls dice itself, which makes games fully reproducible. The engine tracks whose turn it is, where
everybody stands, and who has won.

### Terms

- **Square** - a numbered position `1..size`. Position `0` means "not yet on the board".
- **Snake** - a pair `head -> tail` with `tail < head`. A player who *lands* on `head` is moved to `tail`.
- **Ladder** - a pair `bottom -> top` with `top > bottom`. A player who *lands* on `bottom` is moved to `top`.
- **Last square** - square number `size`. Reaching it ends the game.
- **Exact roll** - a roll that lands precisely on the last square; larger rolls overshoot and are wasted.

## Requirements

1. The board has squares `1..size`. Every player starts **off the board** at position `0`. The first registered player moves first.
2. Players take turns in the order they were registered, cycling back to the first after the last. A turn is one call to `roll(dice)` with a value `1..6`, applied to the player whose turn it is.
3. A player moves forward by the dice value. If the move would pass the last square (position + dice > `size`), the player **stays** where they are (an exact roll is needed to finish) - the turn still passes and `roll` returns the unchanged position.
4. Landing on the **head** of a snake moves the player down to its tail; landing on the **bottom** of a ladder moves the player up to its top. Jumps do not chain: the destination of a jump is final, even if it is itself the head of a snake or the bottom of a ladder. The position reported and stored is the one after the jump.
5. The first player to reach the last square - by walking or by a ladder - wins. After that no more moves are accepted.
6. Dice values outside `1..6` are refused and do not consume the turn.
7. The state is queryable: whose turn it is, each player's position, the winner.

## Constraints

- `size` is between 10 and 10,000; snake heads and ladder bottoms are distinct squares in `2..size-1`; a snake's tail is below its head; a ladder's top is above its bottom.
- 1 to 100 players.

## Examples

Board: `size = 30`, snakes `{27->5, 21->9}`, ladders `{3->22, 8->26}`, players `[a, b]`.

| Call | Who moves | What happens | Returns | Next |
|---|---|---|---|---|
| `roll(3)` | a | 0+3 = 3, ladder bottom -> 22 | `22` | b |
| `roll(1)` | b | 0+1 = 1 | `1` | a |
| `roll(5)` | a | 22+5 = 27, snake head -> 5 | `5` | b |

No chaining (single player `[a]`): `roll(3)` -> 22 (the ladder lands next to the snake at 21, which is ignored because
22 is not 21). `roll(5)` -> 27 is a snake head -> `5`. `roll(3)` -> 8 is a ladder bottom -> `26`.

Overshoot: a on 29 rolls 2 -> 31 > 30, so a stays on 29, `roll` returns `29`, and it is the next player's turn.

Finishing: a on 26 rolls 4 -> 30 = last square. `roll` returns `30`, `winner()` is `a`, `currentPlayer()` is empty,
and any further `roll(1)` returns `-1`.

Invalid dice: `roll(0)`, `roll(7)`, `roll(-2)` all return `-1`; the same player is still to move and nothing moved.

## Edge cases

- Reaching `size` through a ladder top (or any move ending on `size`) wins immediately.
- Landing exactly on `size` is the only way to finish; `size-1` followed by a roll of 1 wins, a roll of 2 does not move.
- `positionOf` of a name that was never registered is `-1` (not `0`).
- After the game is over, `roll` returns `-1` for any dice value and positions stay frozen.

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

The factory receives snakes and ladders as maps from square to destination (head to tail, bottom to top); players are a list of unique names in turn order.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/09-snake-and-ladder test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

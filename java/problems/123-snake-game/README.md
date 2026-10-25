# 123 · Snake Game With Food And Score (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

The classic arcade game, as a deterministic engine without any screen. A snake lives on a rectangular grid. You tell it which way to go, one step at a time; it slides forward, eats the food it runs into (growing longer and scoring a point), and the game ends when it runs into a wall or into itself. There is no randomness: the places where food will appear are given up front as an ordered list, so every game can be replayed move by move.

## Problem

The board has `width` columns and `height` rows. Cells are addressed as `(row, col)`: rows run `0..height-1` from top to bottom, columns `0..width-1` from left to right, and `(0,0)` is the top-left cell. The snake starts with length 1 on `(0,0)`. Each call moves the head by one cell. Food positions come as a fixed ordered list; exactly one food is on the board at a time and the next one appears only after the current one has been eaten. Build the game engine and its observable state.

## Requirements

1. **Creation.** `create(width, height, foods)`: the width and height must be at least 1 (otherwise an error). Every listed food must be a cell inside the board, otherwise the whole creation is an error. The same cell may appear in the list more than once. The game starts with a snake of length 1 on `(0,0)`, score 0, not over, not won. The list given at creation is copied: changing it afterwards has no effect on the game.
2. **Directions.** `"U"` moves the head to `(row-1, col)`, `"D"` to `(row+1, col)`, `"L"` to `(row, col-1)`, `"R"` to `(row, col+1)`. Directions are case-sensitive. Any other input (lower case, longer strings, empty, null) is an error, **also** after the game is over, and changes nothing.
3. **Food and spawning.** The food list is consumed from the front. When the game starts, and again immediately after each food is eaten, the next food is spawned: take entries from the list one by one and discard each entry whose cell is occupied by the snake **at that moment** (after the snake's move has been fully applied, so including the cell just eaten and the tail cell that was kept); the first entry whose cell is free becomes the food. A discarded entry is gone for good, it never comes back later. If the list is used up, there is no food (`"none"`) and the game goes on without food. The food stays where it is until eaten; moving elsewhere never changes it.
4. **A normal move.** If the new head cell is not the food, the head advances and the tail cell is released, so the length does not change. If it is the food, the head advances and the tail is **not** released: the length grows by 1, the score goes up by 1 and the next food is spawned (rule 3).
5. **Walls.** If the new head cell is outside the board the game is over.
6. **Body.** If the new head cell is occupied by a snake cell the game is over. The one exception is the tail cell on a move that does not eat: the tail leaves in the same move, so moving into it is legal.
7. **Reversal.** When the length is greater than 1, moving in the direction opposite to the **last direction that was successfully applied** (`R` after `L`, `L` after `R`, `U` after `D`, `D` after `U`) is treated as moving into the body: the game is over. With length 1 a reversal is an ordinary move.
8. **Fatal moves change nothing else.** On a move that ends the game by wall, body or reversal, the snake is not moved: head, body, length, score and the food shown stay as they were; only the game-over state changes.
9. **Winning.** When the snake occupies every cell of the board (`length == width * height`) the game is over and **won**. This can happen when a food is eaten, and at creation of a 1x1 board (a snake of length 1 already fills it). A won game has no food (`"none"`). The winning move returns -1 like any move that leaves the game over.
10. **`move(direction)` result.** The score after the move, or `-1` if the game is over after this move. That includes the fatal move itself, the winning move, and every move made while the game was already over. Moves made after game over change nothing.
11. **Accessors.** `score()` (foods eaten, always `length() - 1`), `isOver()`, `isWon()` (true only for the full-board ending, never for a wall/body/reversal loss), `length()`, `head()` as `"row,col"`, `body()` as the list of `"row,col"` for every snake cell, **head first**, tail last (a snapshot: later moves do not change a list you already hold), `foodPosition()` as `"row,col"` or `"none"`.

## Constraints

- The game is used by one caller at a time; thread-safety is not required.
- A move must stay cheap however long the snake is (hidden: a 200,000-cell corridor is filled one move at a time, and 400,000 moves are made by a snake chasing its own tail).
- Creating a game must not depend on the board area (hidden: width and height may be as large as 100,000 each).

## Examples

```
create(3, 2, [(1,2), (0,1)])          food at "1,2"
move("R") -> 0     head 0,1
move("D") -> 0     head 1,1
move("R") -> 1     head 1,2  ate the food: body [1,2 ; 1,1]   food now "0,1"
move("U") -> 1     head 0,2  body [0,2 ; 1,2]
move("L") -> 2     head 0,1  ate the food: body [0,1 ; 0,2 ; 1,2]   food now "none"
move("U") -> -1    row -1 is outside the board: isOver() = true, head stays "0,1"
```

```
create(3, 1, [(0,0), (0,2)])          (0,0) is under the snake at creation: skipped, food "0,2"
move("R") -> 0
move("R") -> 1     ate the food, food now "none" (the list is used up)

create(3, 3, [(0,1), (0,1), (0,0), (2,2)])
move("R") -> 1     ate (0,1); the next entries (0,1) and (0,0) are under the snake: both skipped, food "2,2"
```

```
create(3, 3, [(0,1), (1,1), (1,0)])
move "R", "D", "L" eat all three: body [1,0 ; 1,1 ; 0,1 ; 0,0], score 3
move("U") -> 3     head enters 0,0, the cell the tail leaves in this very move: legal, body [0,0 ; 1,0 ; 1,1 ; 0,1]

create(4, 1, [(0,1)])
move("R") -> 1     body [0,1 ; 0,0]
move("L") -> -1    reversal with length > 1: game over, body is still [0,1 ; 0,0]

create(2, 2, [(0,1), (1,1), (1,0)])
move("R") -> 1, move("D") -> 2, move("L") -> -1   the 4th cell is eaten, the board is full: isWon() = true, score() = 3
```

### Edge cases

- Food listed under the snake at the time it would spawn is skipped for good (creation and every later spawn). Listing the same cell twice in a row therefore spawns it once; the second entry is skipped if the snake still covers it.
- The tail is released only on moves that do not eat, so right after eating, the old tail cell is still part of the snake and a listed food on it is skipped.
- A reversal with length 1 is legal; with length 2 or more it ends the game, even though with length 2 the neck is the cell the tail is leaving.
- Moving into the cell the tail is vacating (a snake of length 2 or more circling a block) is legal; moving into any other body cell is not.
- After a fatal move the head stays on the last valid cell; the score is not reduced and the food (if any) stays displayed.
- A direction that is invalid is an error even if the game is over, and it does not count as the "last direction".
- A 1x1 board is already won at creation; every move returns -1.
- A move that eats the last free cell is the winning move and returns -1 (read the score with `score()`).
- An empty food list is valid: the snake just walks around and the score stays 0.

## Glossary

- **Head / tail / body**: the snake is an ordered line of cells; the head is the front, the tail the back, the body is everything in between and including both ends.
- **Spawn**: placing the next food on the board according to rule 3.
- **Reversal**: a move in the direction opposite to the last applied one.
- **Won**: the snake covers the whole board.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Errors: `IllegalArgumentException` for a width or height below 1, a null food list, a food that is null, does not have exactly two numbers or lies outside the board (at creation), and for a direction that is not exactly `"U"`, `"D"`, `"L"` or `"R"` (including `null`).
- `move` returns `-1` whenever the game is over after the call (loss, win, or already over), otherwise the new score.
- Cells are always the string `"row,col"` with no spaces, e.g. `"0,0"`, `"12,7"`; no food is `"none"`.
- `body()` lists head first and always has `length()` entries.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.List;

/** Thin test entry point. Your design lives behind it. Single-threaded: one caller at a time. */
public interface SnakeGame {

    /** "U" (row-1), "D" (row+1), "L" (col-1), "R" (col+1); anything else: IllegalArgumentException.
     *  Returns the score after the move, or -1 if the game is over after this move. */
    int move(String direction);

    int score();                 // foods eaten so far (= length() - 1)
    boolean isOver();            // wall, body, reversal, or the board is full
    boolean isWon();             // over because the snake fills the board
    int length();                // cells occupied by the snake (starts at 1)
    String head();               // "row,col"
    List<String> body();         // every snake cell as "row,col", head first, tail last
    String foodPosition();       // "row,col" or "none"
}
```

Your wiring point (`src/main/java/solution/Solution.java`); each food is an `int[]{row, col}`:

```java
public final class Solution {
    public static SnakeGame create(int width, int height, List<int[]> foods)
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. No clock, randomness or threads are involved. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Initial state, invalid boards, invalid food lists and invalid directions.
- Movement in all four directions, each wall, state after a fatal move, moves after game over.
- Eating, growing, scoring, the food staying put, the tail moving on a normal move.
- Skipping of food under the snake (creation and later), exhausted food list.
- Reversal rules (length 1 vs longer), moving into the leaving tail, hitting the body.
- Winning (2x2 and 1x1 boards), loss vs win, score/length consistency.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/123-snake-game test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`Candidate`): `java/testkit`.

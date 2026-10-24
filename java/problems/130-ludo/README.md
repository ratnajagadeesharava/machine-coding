# 130 · Ludo (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 2 - the entry point and the tests are provided; the board, the rules engine and its data model are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

The classic race game for two to four players, played entirely through code. Each player has four tokens that start in a yard, must be released with a six, travel once around a shared circular track, turn into a private home column and finish on an exact count. Landing on an opponent's lone token sends it back to the yard. There is no randomness inside your program: the dice are handed in when the game is created and are used strictly in order, so every game is a deterministic story that can be replayed and checked move by move. Your program decides whose turn it is, which tokens may move for a roll, what a move does, and when the game is over.

## Requirements

### Players, colours, tokens

- A game has 2, 3 or 4 players. The colours, in turn order, are `RED`, `GREEN`, `YELLOW`, `BLUE`; a game with N players uses the first N of them. `RED` always plays first, and play continues in that order, wrapping around.
- Every player has 4 tokens, numbered 0..3. A token's **progress** says where it is:

| Progress | Position |
|---:|---|
| 0 | in the yard (not on the board) |
| 1 | on the player's own **start cell** |
| 2 .. 51 | on the shared track, one cell further clockwise per step |
| 52 .. 56 | in the player's private **home column** (five cells) |
| 57 | **HOME** (finished) |

### The track

- The shared track has 52 cells numbered 0..51 clockwise; after cell 51 comes cell 0.
- Start cells: `RED` 0, `GREEN` 13, `YELLOW` 26, `BLUE` 39.
- A token of colour *c* with progress *p* in 1..51 stands on cell `(start(c) + p - 1) mod 52`. So progress 1 is the start cell and progress 51 is the cell two before it: a colour never visits the cell immediately before its own start cell (it turns into its home column one cell early). Progress 52 or more is off the shared track and can never be captured and never captures.
- **Safe cells**: the four start cells (0, 13, 26, 39) and the cell 8 after each of them (8, 21, 34, 47). A token landing on a safe cell never captures anyone there. Tokens of different colours may share a safe cell.

### A turn

- State at any time: whose turn it is, and whether a roll is waiting for a move.
- `roll()` takes the next number from the dice list given at creation and returns it. It belongs to the current player.
- A token **can move** with a roll *v* when:
  - it is in the yard and *v* is 6 (it goes to progress 1), or
  - it is on the board (progress 1..56) and `progress + v <= 57`. Reaching 57 needs the exact count: a roll that would go past HOME does not allow that token to move. A token at progress 57 never moves.
- If at least one token can move, the roll waits for `move(tokenIndex)`; `legalMoves()` lists the indexes of the tokens that can move, ascending. If no token can move, the roll is simply used up and nothing is asked of the player.
- `move(tokenIndex)` moves that token by the rolled value (a yard token goes to progress 1), applies capturing, and resolves who plays next.
- **Who plays next** after a roll has been used (by a move, or because nothing could move): the same player rolls again if the roll was a 6, or if the move captured something, or if the move brought a token HOME; otherwise the next colour in turn order plays. A 6 gives the extra turn even when nothing could move with it.
- **Three sixes**: if the current player rolls a 6 as the third roll in a row (consecutive rolls of this player, with no non-6 roll in between and no other player in between), that roll is forfeited: `roll()` still returns 6, nothing moves, no choice is offered, and the turn passes to the next colour. The count starts again from zero when the turn passes to someone else or when a non-6 is rolled. (A 6 that earned an extra turn, a capture or a HOME arrival counts like any other 6; a capture or HOME reached with a non-6 does not interrupt anything because a non-6 has already restarted the count.)

### Capturing

- A move that ends on a shared cell (progress 1..51) which is **not** a safe cell may capture. Look at each opponent colour separately, in turn order: if exactly **one** token of that colour stands on the cell, it goes back to the yard (progress 0). If **two or more** tokens of the same colour stand on the cell they form a **block** and are not captured. Your own tokens are never affected, and tokens of the same colour may share any cell.
- A capture gives the capturing player an extra turn (see above). Several colours can be captured by one move.
- Blocks and safe cells only protect against capture; they never stop a token from moving through or landing on a cell.

### Winning

- The first player with all four tokens at progress 57 wins. The game ends immediately with that move: no further roll or move is accepted, `isOver()` is true, `winner()` is that colour and `currentPlayer()` keeps returning the winner.

### The dice

- The list is given at creation (values 1..6) and consumed one value per `roll()`. When it is used up, `roll()` fails with "dice exhausted" and changes nothing. A game does not have to use all of its dice.

## Behavioural contract (entry point)

```java
// solution.Solution.create(int players, List<Integer> dice) -> contract.LudoGame
int          roll();                         // next die for the current player (1..6)
List<Integer> legalMoves();                  // ascending; empty if no roll is pending
String       move(int tokenIndex);           // "moved" | "home" | "captured:<COLOUR>:<n>[,captured:...]"
String       currentPlayer();                // "RED" | "GREEN" | "YELLOW" | "BLUE" (the winner once the game is over)
int          tokenProgress(String colour, int tokenIndex); // 0..57
int          tokenCell(String colour, int tokenIndex);     // 0..51 on the shared track, else -1
Optional<String> winner();
boolean      isOver();
```

Errors: `IllegalStateException` for `roll()` when the game is over, a move is pending or the dice are used up (message `dice exhausted`), and for `move()` when the game is over or no roll is pending; `IllegalArgumentException` for a token that is not legal (including an index outside 0..3), for an unknown colour or token in the queries, and for invalid creation arguments. The full Javadoc is in `src/main/java/contract/LudoGame.java`.

Move descriptions: `"moved"` for an ordinary move or leaving the yard, `"home"` when the token reaches progress 57 (also when this wins the game), `"captured:<COLOUR>:<n>"` when something was captured, where `<n>` is the number of tokens of that colour sent back (always 1 under the block rule). If several colours are captured by the same move the entries are joined with `","` in turn order, for example `"captured:RED:1,captured:YELLOW:1"`. A capture that happens on a move that also reaches HOME cannot occur (home is never a shared cell).

## Examples

`dice` is the list given at creation. Colours not mentioned have all four tokens in the yard.

### 1. Leaving the yard, a pass, and a capture

Two players, dice `6 6 5 6 2 4 3`.

| Roll | Player | Value | `legalMoves()` | Call | Result |
|---:|---|---:|---|---|---|
| 1 | RED | 6 | 0,1,2,3 | `move(0)` | `moved`, RED 0 at progress 1 (cell 0). RED rolls again |
| 2 | RED | 6 | 0,1,2,3 | `move(0)` | `moved`, progress 7. RED rolls again |
| 3 | RED | 5 | 0 | `move(0)` | `moved`, progress 12 (cell 11). GREEN's turn |
| 4 | GREEN | 6 | 0,1,2,3 | `move(0)` | `moved`, progress 1 (cell 13). GREEN rolls again |
| 5 | GREEN | 2 | 0 | `move(0)` | `moved`, progress 3 (cell 15). RED's turn |
| 6 | RED | 4 | 0 | `move(0)` | `captured:GREEN:1`: RED 0 at progress 16 (cell 15), GREEN 0 back at 0. RED rolls again |
| 7 | RED | 3 | 0 | `move(0)` | `moved`, progress 19. GREEN's turn |

If instead the first die had been a 3, `roll()` would return 3, `legalMoves()` would be empty and it would already be GREEN's turn.

### 2. The third six is forfeited

Two players, dice `6 6 6 2`: RED rolls 6 and moves token 0 to progress 1, rolls 6 again and moves it to 7, rolls a third 6: `roll()` returns 6, `legalMoves()` is empty, RED 0 is still at progress 7 and it is GREEN's turn. GREEN rolls 2: nothing can move, so it is RED's turn again.

### 3. Exact finish

Two players, dice `6 6 4 1  6 6 4 1  6 6 4 1  6 6 3 1  2`. RED always moves token 0; GREEN never leaves the yard (it only rolls 1s). Each group `6 6 4` takes token 0 to progress 11, 27, 43 (cell 42) and then, in the fourth group, the 6s take it to 49 and 55 (a home-column cell, `tokenCell` is -1). The `3` would need 58: nothing can move and GREEN plays (rolling 1). Back to RED, the `2` is exact: `move(0)` returns `home`, progress 57, and RED rolls again.

### 4. What is protected

Two players. After each list the last move is by RED and returns `moved` (no capture):

| Dice | Moves played (token index per move, in order) | Why nothing is captured |
|---|---|---|
| `6 4 6 2 5 6 2 6` | 0,0,0,0,0,1,1,0 | GREEN tokens 0 and 1 both stand at progress 3 (cell 15) = a block; RED token 0 lands there (progress 16) |
| `6 4 6 4 5 4 6 6` | 1,1,1,1,1,1,1,1 | GREEN 1 stands on cell 21 (progress 9), a safe cell; RED 1 lands there (progress 22) |
| `6 3 6 6 1 6 4` | 0,0,0,3,0,0,0 | GREEN 3 stands on cell 13, GREEN's start cell (progress 1); RED 0 lands there (progress 14) |

In all three, every token involved is still where it was and the two colours simply share the cell.

### 5. Three complete games

The moves are chosen by a fixed rule so that only the dice matter. A seat's rule is **lead** (play the legal token with the highest progress, ties to the lowest index) or **first** (the lowest legal index). After each roll that offers a choice, the rule picks the token; rolls that offer none just pass.

**Game A** - 2 players (RED: lead, GREEN: first), 77 dice. GREEN only ever rolls 1s and never leaves the yard.

```
3 1 6 4 1 6 4 1 6 6 3 1 4 1 5 1 6 6 2 1 6 6 5 1 6 5 1 2 1 6 6 5 1 6 6 3 1 6
4 1 6 3 1 2 2 6 2 1 6 4 1 6 6 4 1 3 1 4 1 6 6 3 1 6 2 6 6 6 1 5 1 6 6 6 1 6
6
```

RED token 0 reaches HOME at roll 44 and token 1 at roll 45; token 2 at roll 65. Rolls 68 and 74 are forfeited third sixes. Token 3 reaches HOME at roll 77 and RED wins. No captures. Final progress: RED 57 57 57 57, GREEN 0 0 0 0.

**Game B** - 2 players (RED: lead, GREEN: first), 96 dice.

```
4 6 6 6 4 2 6 4 2 4 1 4 4 5 6 2 4 4 6 6 5 4 6 3 3 3 4 6 4 6 6 3 6 5 2 6 6 6
3 3 4 5 1 5 5 5 2 4 5 5 6 3 3 6 3 5 3 4 4 3 3 6 6 5 2 4 4 6 3 6 5 6 5 6 6 3
6 4 2 6 5 3 4 5 5 5 6 5 5 3 6 6 6 5 6 3
```

Roll 4 is GREEN's forfeited third six. RED captures a GREEN token at roll 23 and again at roll 50 (both `captured:GREEN:1`). RED tokens reach HOME at rolls 28, 55, 90 and 96; rolls 38 and 93 are forfeited sixes. RED wins at roll 96. Final progress: RED 57 57 57 57, GREEN 46 0 0 0.

**Game C** - 4 players (RED: lead, GREEN: first, YELLOW: lead, BLUE: first), 142 dice.

```
4 1 4 2 1 5 6 5 3 6 4 3 6 4 6 6 4 5 3 4 3 3 6 4 5 5 3 2 6 6 4 5 5 3 6 4 3 4
6 4 6 6 4 5 2 1 6 6 4 4 4 2 4 3 4 3 6 4 2 4 5 6 6 4 2 2 2 6 6 6 2 3 3 6 6 6
6 1 1 5 6 3 2 2 2 6 6 4 3 6 2 6 3 6 4 1 3 6 2 5 4 6 4 2 4 5 6 6 6 4 4 4 6 4
3 3 4 4 2 5 6 2 6 3 4 1 5 6 3 5 4 1 5 3 4 6 2 6 3 6 6 4
```

YELLOW captures a RED token at roll 35, RED captures a GREEN token at roll 93 and YELLOW captures a BLUE token at roll 139 (`captured:RED:1`, `captured:GREEN:1`, `captured:BLUE:1` in that order). YELLOW tokens reach HOME at rolls 41, 68, 107 and 142; rolls 70, 76 and 109 are forfeited sixes. BLUE's token 0 reaches HOME at roll 89. YELLOW wins at roll 142. Final progress: YELLOW 57 57 57 57, RED 44 0 0 0, GREEN 17 0 0 0, BLUE 57 0 0 0.

## Edge cases

- `roll()` while a move is pending, `move()` with no roll pending (including right after a roll that was resolved automatically), a token that cannot move with the pending roll, a token index outside 0..3, and anything after the game is over are all errors. A rejected call changes nothing: the pending roll stays pending and no die is consumed.
- Exhausted dice are an error only when a roll is attempted; a pending move can still be played after the last die was used.
- Invalid creation: players outside 2..4, a null list, or any die outside 1..6.
- Queries (`tokenProgress`, `tokenCell`) reject colours that are not in the game (including lower-case names) and token indexes outside 0..3.
- A 6 with no legal move (every token is in the home column or HOME) still gives the extra turn.
- Several tokens of one colour can stand in the yard, on the same cell, or at HOME; each is tracked separately.
- Moving onto your own token never captures; landing on an opponent block or on a safe cell is an ordinary move.
- A capture or a HOME arrival on a 6 keeps counting that 6 towards the three-sixes rule.

## Glossary

- **Yard**: where tokens wait before entering; progress 0.
- **Start cell**: the shared cell a colour's token lands on when it leaves the yard.
- **Home column**: five private cells (progress 52..56) after the shared track; **HOME** is progress 57.
- **Block**: two or more tokens of one colour on the same shared cell.
- **Extra turn**: the same player rolls again.

## Running

```bash
cd java/problems/130-ludo
mvn test                 # behavioural tests (skipped until Solution.create is wired)
STRICT=1 mvn test        # "not implemented" becomes a failure
```

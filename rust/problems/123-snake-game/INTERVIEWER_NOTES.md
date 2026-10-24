# Interviewer notes — 123 · Snake Game With Food And Score

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- A 100,000 x 100,000 board: allocating a grid of `width * height` cells fails or hangs; occupancy must be sparse.
- A 200,000-cell corridor eaten cell by cell: a list scan per move (`body.contains(newHead)`) is quadratic and times out.
- 400,000 moves of a length-4 snake circling a 2x2 block: the leaving-tail exception must work forever, not only once.
- Eating while the next listed food is on the old tail cell: the tail is not released on an eating move, so it is skipped.
- A run of skipped foods in the list, including repeats of the same cell, ending in a free one.
- A rejected direction (`"l"`) between valid moves must not clear or overwrite the last direction used for the reversal rule.
- The caller mutating its food list (or the `int[]` cells) after `create`: the game must have copied them.
- `body()` must be a snapshot, not a live view.
- A move that eats the last cell returns -1 although the score went up; `score()` is the way to read it.

## Likely interviewer questions
- Why is the order of checks what it is? What changes if the body check happens before the wall check? (nothing observable) Before the reversal check? (length 2 differs)
- Why is the tail cell a legal target on a non-eating move but not on an eating one?
- What data structure do you use for the body and for "is this cell occupied", and what does each move cost?
- Is the reversal rule a special case, or does it follow from another rule? (It does not for length 2: the neck is the leaving tail, so the explicit rule is needed.)
- When exactly is a food skipped, and why does that not make the game non-deterministic?
- What does the fatal move leave behind, and why did we choose "snake does not move"?

## Invariants
- `length == body.size == occupied.size`; `score == length - 1`.
- The food is never on a snake cell at the moment it is spawned, and there is at most one food.
- The skipped food entries are never revisited; the cursor only moves forward.
- After game over (loss or win) nothing but the error check for the direction does anything.
- `won` implies `over`; `won` is true only if `length == width * height`.

## Common mistakes
- Removing the tail before the body check, which makes the cell legal on an eating move too (or checking after removal and forgetting to re-add it on a failed move).
- Never releasing the tail from the occupancy set, so the snake can never circle its own tail.
- Releasing the tail on an eating move (the snake does not grow).
- Spawning the food before the move is applied, so it can land under the new head or the kept tail.
- Treating a skipped food as "eaten" (score changes) or putting it back later.
- Row and column swapped (`(x, y)` thinking): food lists are `(row, col)`, `width` counts columns.
- Treating the reversal as legal for length 2 because "the tail is leaving" (contradicts requirement 7).
- Updating the "last direction" on an invalid or fatal move.
- Returning the score instead of -1 on the winning move, or reporting a win for a loss.
- `int` overflow in `width * height` for the win check.
- Case-insensitive directions, or silently ignoring an unknown direction.

## Over-engineering traps
- A `Direction` class hierarchy with one subclass per direction before an enum or a delta table is considered.
- A `Cell`/`Position` value type is fine; an `Observer` or event bus for "game over" and "food eaten" is not asked for.
- A full state machine framework for Playing/Won/Lost when two booleans suffice (but a small enum is good).
- A strategy per movement rule before a second rule exists (wrap-around, V5, is the justification).

## Design and scaling discussion
- Deque plus hash set gives O(1) per move; a 2D boolean grid is also O(1) but costs O(W*H) memory at creation. Which one for which board sizes?
- Encoding a cell as a single `long` (`row * width + col`) vs a record: allocation and hash quality.
- Where do rules live so that obstacles, wrap-around and two snakes are additions: a rule/strategy per movement concern (wall policy, collision policy, food policy) is a good place for a pattern to appear, but only once V2/V5 asks for variants.
- State pattern for Playing/Over/Won: acceptable, but the moves-after-game-over rule is a single early return.
- Determinism: all randomness lives in the (given) food list, which makes replay, tests and bug reports trivial; a random food spawner would be a collaborator injected behind an interface and driven by a seeded sequence in tests.
- A real game loop would add a tick source and input queue; the engine stays pure.

## Trade-off questions to ask when the candidate is done
- Fatal move leaves the snake in place vs moves the head into the wall: which is easier for a renderer?
- Returning -1 for both "lost" and "won": what would you change in the API to remove the ambiguity?
- How would you support a free-cell lookup for a random food spawner on a big board with a long snake?
- How would you test the win path on a 1000 x 1000 board without writing a million moves by hand?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.

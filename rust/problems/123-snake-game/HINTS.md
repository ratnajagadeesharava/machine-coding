# Hints — 123 · Snake Game With Food And Score (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Write the sequence of questions one move asks, in order: which of them can end the move early, and which of them change state? Then ask which cells are "occupied" at the exact moment you test the new head, and which at the moment you spawn the next food.

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

The snake is an ordered line where you add at one end and remove at the other, and you also need to ask "is this cell part of the snake?" in constant time. Keep both views of the same cells and update them together. The four directions are four row/column deltas, so the movement rule can be data rather than four branches. The food list is a cursor into the list.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `Game { width, height, snake: deque of cells (head at the front), occupied: set of cells, foods + next index, current food (optional), last direction, over, won }`. A move validates the direction, returns -1 if already over, rejects a reversal when `length > 1`, computes the new head, checks the wall, decides `eats = newHead == food`, and only then checks the body: against `occupied` minus the tail when not eating (the tail is about to leave). Apply: push the new head; pop the tail and remove it from `occupied` only when not eating; when eating, increase the score and spawn the next food after the snake is updated (advance the cursor while the entry is occupied). Win when `length == width * height` (use 64-bit arithmetic).

**Java:** `ArrayDeque<Integer>` of encoded cells (`row * width + col` as a `long` if the board can be huge) plus a `HashSet<Long>`; copy the incoming `List<int[]>` (including the arrays) at creation.

**Rust:** `VecDeque<(i32, i32)>` plus a `HashSet<(i32, i32)>`; `foods.to_vec()` at creation, a `usize` cursor, and an `Option<(i32, i32)>` for the food.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

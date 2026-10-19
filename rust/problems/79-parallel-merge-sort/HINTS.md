# Hints — 79 Parallel Merge Sort (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Two threads sorting two halves of the same array - do they ever touch the same element? What must be true before you merge?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Split the range in two; sort the halves in parallel; wait for both; merge through a temporary buffer. Below a threshold (thousands of elements) just sort sequentially. The wait is also your synchronisation: it makes the halves' writes visible.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Recursive task `sort(lo, hi, depthBudget)`: if the range is small or the thread budget is used up, sort sequentially; else fork the left half, sort the right half yourself, join the left, then merge both through a scratch array of the same length (allocated once). Cap threads by passing a budget that halves at each level (Rust: `thread::scope` + `split_at_mut`) or by running on a pool/`ForkJoinPool` of size `parallelism` (Java). Do not use a fixed-size ordinary thread pool for the recursion - why would it hang?

**Rust:** `std::thread::scope` lets you borrow `&mut` sub-slices from `split_at_mut` into spawned threads with no `Arc`, `Mutex` or `unsafe`.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

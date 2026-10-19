# Hints — 80 Parallel Word Count (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

If every thread does `map[word] += 1` on the same map, what does the line really do in three steps? How could two threads interleave?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Let each worker count into its **own** private map and merge the maps once at the end. Contention disappears and no locks are needed while counting.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Partition the documents (or one document's text) into `parallelism` chunks; each worker tokenises its chunk into a private map; a final step adds the partial maps (do it sequentially, or in a tree). For a split inside a document, move the cut forward to the next separator so a word never straddles two chunks. If you prefer a shared concurrent map, use its atomic `merge`/`entry` operation - never get-then-put.

**Rust:** `std::thread::scope` with each worker returning its own `HashMap`; merge on the main thread. Chunks of `&[String]` or `&str` can be borrowed without cloning.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

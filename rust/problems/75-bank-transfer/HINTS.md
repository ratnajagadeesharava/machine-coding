# Hints — 75 Bank Transfers Without Deadlock (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Draw two threads, A->B and B->A. In which order does each take its two locks? What would make every thread take them in the *same* order?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

A global order on accounts (an id, a creation number, the name) lets every transfer lock the lower one first. Do the balance check while holding both. For `total()` think about what it has to hold to be a snapshot.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Give each account a lock and a unique, comparable id. `transfer`: look up both accounts, lock them in id order, check funds, update both, unlock in any order. `total()`: either lock *all* accounts in id order (consistent), or use one read-write lock where transfers take the read side... but then think about whether two disjoint transfers may still overlap. Validate arguments before taking locks.

**Rust:** `Mutex<u64>` per account inside an `Arc`, ordered by an id field. Keep the guards alive until the update is done; mind which guard is `from` and which is `to` after you ordered them.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

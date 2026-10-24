# Hints — 137 Configurable Time Window Hit Counter (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Hits that happened in the same millisecond (or the same short stretch of time) are indistinguishable for every question you are asked. Do you have to remember them one by one? And when does a stored piece of data stop mattering?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Store, per key, ordered pairs of (time, how many) instead of single hits, merging hits that share a time (or a time bucket). Expired pairs are always the oldest ones, so they can be removed from one end whenever the key is touched. Think about what else must trigger removal if a key is never touched again, and about which single lock-protected value gives you the "never go back in time" clamp.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `Counter { window, lastSeen, keys: map<String, Series> }` where `Series` is a queue of `(timestamp, count)` ordered by timestamp, with consecutive equal timestamps merged. Every operation first computes `now = max(clock, lastSeen)`, stores it, evicts from the front of the series it touches while `ts <= now - window`, then answers. `countInLast` walks back from the newest end (or keeps a running total for the full window and subtracts what is older than the duration). `setWindow` evicts with the old window first, then with the new one. A circular array of fixed buckets is the alternative when you can accept a coarser time resolution; then the bucket length is a trade-off between precision and memory.

**Rust:** `Mutex<State>` with `HashMap<String, VecDeque<(u64, u64)>>`; shard the map (`Vec<Mutex<..>>` by key hash) only once the single lock is correct. Keep the last seen time inside the same lock as the data so the clamp and the insert are one step.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

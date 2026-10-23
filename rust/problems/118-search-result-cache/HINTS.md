# Hints — 118 Search Result Cache (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Take one `get` and list the questions in order: what is the key? Is there a usable stored answer? If not, is someone already fetching it? Which of those answers must be decided together, and which step must happen without holding that decision's lock?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Two maps: one for stored entries (with a recency order for eviction) and one for calls that are in flight. A stored entry carries the results, the time it was stored and the index version read before its call. The first thread to miss registers an in-flight call and goes to the backend with the lock released; the others find the registration and wait on it. Whoever finishes removes the registration, stores the entry (only if it is still the registered call) and wakes the waiters.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `get` = normalise, read `indexVersion`, lock; if an entry is live (age < ttl and same version) record the hit, mark it most recent, return a copy; otherwise drop it, count a miss, and either join the in-flight call for the key or create one and become its leader; unlock. The leader calls the backend, copies the list, locks, and if its call is still the registered one removes it and inserts the entry (evicting the least recently used while over capacity), unlocks, and completes the shared result (value or failure). On a failure nothing is inserted but the registration is still removed. `invalidate*` remove entries **and** registrations. `size()` counts only live entries.

**Rust:** `Mutex<State>` plus a `Condvar` per in-flight call is enough; never hold the state lock while calling the backend. A `VecDeque` of keys or a counter-stamped map are both fine for recency.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

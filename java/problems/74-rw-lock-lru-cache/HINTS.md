# Hints — 74 Concurrent LRU Cache (Read-Write Lock) (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Which of the operations really only *read* shared state, and which ones secretly write (hint: look at what `get` does to the order)?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

A read-write lock only helps the operations that truly do not modify anything: `peek`, `size`. `get`, `put` and `remove` change the order or the map, so they take the write side. Keep the map and the recency list consistent inside one critical section.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Use a hash map from key to node plus a doubly linked list (head = most recent, tail = least recent), all guarded by one `ReadWriteLock`. `peek`/`size`: read lock, no mutation (careful: some library maps - e.g. an access-ordered `LinkedHashMap` - mutate on `get`). `get`/`put`/`remove`: write lock, unlink + push front, evict from the tail when `size > capacity` *before* returning. Never call a write operation while holding the read lock.

**Java:** `ReentrantReadWriteLock`. An access-ordered `LinkedHashMap` is tempting but its `get()` mutates the structure - using it under a read lock is a data race.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

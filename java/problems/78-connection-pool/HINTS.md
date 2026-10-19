# Hints — 78 Bounded Connection Pool (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

What are you waiting *for*, and who tells you it has happened? What must be true when you wake up before you may return a connection?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Two classic shapes: a semaphore with `size` permits plus a free-list, or a lock with a condition and a free-list. Whichever you choose, compute the deadline once and re-check the state after every wake-up.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Keep a free-id queue and a `leased[]` flag array under one lock with one condition. `acquire`: compute `deadline`; loop { if closed -> fail; if a free id exists -> mark leased, return it; if the deadline has passed -> return nothing; wait for the remaining time }. `release`: validate (range, leased), mark free, push the id, wake a waiter. `close`: set closed, wake all. With semaphores, think hard about what `close` and `double release` do to the permit count.

**Java:** `ReentrantLock` + `Condition.awaitNanos(remaining)` returns the remaining time for you. `Semaphore.tryAcquire(timeout)` is fine for the permit but cannot tell you which id to hand out.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

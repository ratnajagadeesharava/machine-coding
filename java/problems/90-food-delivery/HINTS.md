# Hints — 90 Food Delivery Dispatch (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Which events can make an order and a free partner meet, and does your design cover every one of them?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Model the order as a small state machine where each transition is a guarded method. For partners keep a set of free ones and a FIFO of waiting orders; the same 'try to dispatch' routine has to run after a ready, a delivery and a cancel. Everything that touches stock, orders and partner state must be consistent in one critical section.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Keep per-restaurant stock maps and a global order table. `placeOrder` validates every line, checks every stock level, then decrements all. `markReady` calls `dispatch()`: while the queue is non-empty and a free partner exists, pop the oldest waiting order and give it the free partner with the minimum `(distance, id)`. Call `dispatch()` at the end of `deliver` and of `cancel`. Cancel also needs to remove the order from the queue if it is waiting, release its partner if it has one, and add its quantities back.

**Java:** A single `ReentrantLock`/`synchronized` guarding stock, orders, free partners and the waiting `ArrayDeque` is the safe start; `Optional<String>` from `partnerOf` must be a snapshot taken under the lock. `Map.of` has no stable order, so validate every line before mutating.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

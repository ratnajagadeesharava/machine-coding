# Hints — 112 Parking Lot with Allocation Strategies (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Write down the two questions `park` must answer: "which spots are free for this vehicle type?" and "which one of them do I pick?". Which of the two changes when the strategy changes, and which one does not?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Keep the free spots in a structure that can hand out the lowest index quickly and take a freed index back, separately per vehicle type (and per floor if your strategy needs floor counts). The strategy only looks at those structures and returns a spot; the lot does the booking. Tickets and registrations are two indexes onto the same parking record, and all of it must change under one guard.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: for each (type, floor) a sorted set of free global indices; a map ticket -> (type, floor, spot, registration) and a set/map of parked registrations; a counter for ticket ids. `LOWEST_INDEX_SPOT` takes the smallest first-element over floors of that type, `MOST_FREE_FLOOR` takes the floor with the largest set (smallest floor number on ties) and its first element. `park` checks the registration, picks, then removes from the free set and records the ticket in one critical section; `unpark` reverses it.

**Java:** `TreeSet<Integer>` per (type, floor) gives `first()`/`pollFirst()`/`add` in O(log n); a single lock (`synchronized` or `ReentrantLock`) around `park`/`unpark`/`setStrategy` is a sound start; make the strategy `volatile` or guard it with the same lock.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

# Hints — 138 Multi-Threaded Page Visit Counter (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Which facts must be updated together for a single visit so that no reader can ever see them disagree, and which facts about different pages never need to agree except when someone asks for a snapshot?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Make the normalised url the only key anywhere, and compute it once at the door of every operation. For the counts, think about what a visit really changes: one number for the page and one number for the (page, visitor) pair, so memory grows with pairs and not with visits. For the snapshot, decide who is allowed to run during it: either everybody else waits, or you take a version of the data that cannot change under you. Choose the simpler of the two first.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `pages: normalised url -> Page { total, visitors: id -> count }`. A visit finds-or-creates its page and bumps both numbers. Visits take a shared "writers" permission and the snapshot takes the exclusive one, so it observes a state between two visits; it then copies and sorts. `reset` removes the page entry under the same shared permission. Validate before touching any map so a rejected call leaves nothing behind.

**Java:** `ConcurrentHashMap` with `computeIfAbsent` for first sight and `LongAdder` (or `AtomicLong`) for the counts; a `ReentrantReadWriteLock` whose read side is used by writers is a perfectly good snapshot gate. Strip and lower-case with `String` methods, no regex needed.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

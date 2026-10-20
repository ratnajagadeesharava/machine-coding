# Hints — 88 Shared Event Calendar (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

What does 'nothing is created' imply about the order in which you check and write: can you decide the outcome for ALL participants before you change any single calendar?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Per user you need to answer 'does anything overlap [s,e)?' fast, so think of an ordered structure keyed by start time per person. Because several people are involved in one booking, think about which lock(s) protect a person's calendar and in which order several of them may safely be taken.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Keep a per-user sorted set of events and an id-to-event index. To book: collect all participants, order them in one global canonical way (for example by name), lock them in that order, check overlap against each sorted set (look only at the neighbours of the new start), then insert into all or into none and release. A recurring series is the same operation over the union of all occurrences x all participants under the same lock set. `commonFreeSlots` is a merge of the users' busy intervals inside the window followed by a gap scan.

**Java:** `TreeMap<Long, Event>` per user with `floorEntry`/`ceilingEntry` for the neighbour check; per-user `ReentrantLock` taken in sorted-name order, or one coarse lock first if you are short of time and then refine.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

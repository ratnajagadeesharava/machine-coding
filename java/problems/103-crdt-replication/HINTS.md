# Hints — 103 CRDT Replicated Store (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

If the same snapshot arrives twice, or after a newer one, what property must the combine operation have so that nothing changes the second time?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

For each type look for a state shape whose merge is 'per component, take the larger / the union': the counter needs one entry per replica, the set needs to tell additions apart (a unique tag per addition) and remember which tags were removed, the register just keeps the best (timestamp, id).

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Counter: two per-replica maps (increments, decrements), merged by max per replica id. Set: a collection of tagged additions and a collection of removed tags, each merged by union; present = added and not removed. Register: the larger (timestamp, replica id). Serialize all three with escaping so any string survives, and make sure `snapshot` copies under the same lock that updates use.

**Java:** Escape or length-prefix the strings in your snapshot (names can contain newlines and separators). `Collectors`/`TreeMap` give deterministic output; copy under a lock (or `synchronized`) so `snapshot` is consistent.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

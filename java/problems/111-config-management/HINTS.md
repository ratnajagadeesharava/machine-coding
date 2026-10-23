# Hints — 111 Configuration Management Service (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Which single piece of information must be compared and changed in one indivisible step for two racing updaters to never both succeed? And what should be true about the state when a call fails halfway?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Keep, per config, the whole list of versions as the single source of truth: the current record is just its last entry and the version number is its length. Do all the checks (validation first, then existence, then version) before you touch anything, and make the write of the new entry one atomic step with the version comparison. Build the display string only when a caller asks for it.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `Entry { value, version, updatedAt }` as an immutable value; a config owns an append-only list of entries; the store maps namespace to a sorted map from key to config, so `list` is just an in-order walk. A change either appends exactly one entry or throws/returns false before appending anything. Rollback is "append an entry whose value is copied from entry N".

**Java:** `ConcurrentHashMap<String, ConcurrentSkipListMap<String, ...>>` or a per-namespace lock around a `TreeMap`; `compute` on the map entry gives you an atomic check-and-append. Return `List.copyOf(...)` snapshots from `history` and `list`. Validate keys with a precompiled `Pattern` (or a char loop), and read `clock.millis()` inside the atomic section.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

# Hints — 122 Versioned Value Store (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Ignore concurrency and compaction at first. For one key, what do you need to remember so that you can answer "what was the value when the store was at number 7"? And which single piece of state is shared by all keys?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Keep, per key, a list of entries ordered by version (a delete is an entry too), and one counter for the whole store. The step "take the next number and make the entry visible" must be indivisible with respect to every other write and every reader. A batch is one such step. A snapshot only needs to remember a number; compaction must then look at the smallest numbers still in use.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `Store { version counter, floor, entries per key (sorted by version, tombstone = no value), open snapshots (id -> version) }`. `getAt` = binary search for the last entry with version <= asked. `putIfVersion` = look at the key's last entry (live? then its version, otherwise 0) and append in the same critical section. `compact(F)` = for each key keep the entries >= F plus the newest entry at or before F and at or before each open snapshot's version, then drop a leading tombstone.

**Rust:** `RwLock<Inner>` with `HashMap<String, Vec<Entry>>` is enough to start; `slice::partition_point` on the version finds the entry. Use `Option<String>` for the value so a tombstone is `None`.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

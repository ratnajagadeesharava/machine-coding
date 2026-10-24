# Hints — 134 Dictionary App (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Almost every operation starts the same way: turn the caller's text into the key you store. Where should that happen, and what must happen before you change anything?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Think about what order the answers must come back in. If the words are kept in that order, prefix queries, the sorted lists and the word of the day all become cheap. For edit-distance 1, ask which strings can be one edit away from a given string, and what that means for how many words you must compare.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: a sorted map from the normalised word to a small record holding an ordered list of `(part of speech, text)` entries and a sorted set of synonym names; one read-write lock around the map (reads share, edits are exclusive) is enough to start with. `prefix` is a range scan starting at the prefix key and stopping at the first word that no longer starts with it. Removing a word walks its synonym set and removes the back links. For `suggest`, comparing the input with each word using a "within one edit" check that stops early on length difference is enough; a smarter index is a follow-up.

**Rust:** `RwLock<BTreeMap<String, WordRecord>>` with `range(prefix.to_string()..)` for the prefix scan; `i64::rem_euclid` for the word of the day. Validate and normalise first, and only then take the write lock, so a failed call leaves no trace. Return owned `Vec<String>`s, never guards.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

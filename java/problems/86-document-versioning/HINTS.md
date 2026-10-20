# Hints — 86 Document Versioning with Merge (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

When an edit arrives for an old version, what three texts do you have, and what would you compare to know what each side actually changed?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Treat a version as a list of lines. A line diff (longest common subsequence) turns 'base to latest' and 'base to mine' into lists of changes at positions of the base. Merging is combining two lists of non-overlapping changes; conflict detection is about overlapping positions. Keep 'is my base current, then append' in one critical section.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `Doc { versions: Vec<(author, lines)> }` per document, guarded so an edit reads latest, merges and appends atomically. For the merge, convert each diff into hunks with `(start, end, replacement)` relative to base lines, sort both lists, walk them together, reject overlaps and same-point insertions, apply identical hunks once, and apply the rest from the end to the start so offsets stay valid. For diff, trim the common prefix and suffix before running the LCS.

**Java:** `split("\n", -1)` keeps trailing empty lines; `String.join("\n", lines)` reverses it. A `ReentrantLock` per document (or `synchronized (doc)`) around read-merge-append is enough; the concurrency tests only need atomic compare-and-append.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

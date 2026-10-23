# Hints — 109 Document Line Search (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

The document never changes except for deletions, but it is searched many times. What work can you do once, at load time, so that a search does not have to look at lines that cannot match?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Turn each line into its set of words once, and keep for every word the ordered ids of the lines that contain it. A query then becomes "combine a few ordered id lists" instead of "scan the document". Treat a deletion as a fact you check about an id, not as an edit of those lists.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `word -> sorted ids` built once with one tokenizer shared by lines and queries; a search splits and lower-cases the query, drops duplicate words, returns empty if there are none or if any word is unknown, intersects the lists (start from the shortest), and filters out ids that are in the deleted set. `deleteLine` checks the id range and flips the id into the deleted set in one atomic step, returning whether it was the one that flipped it.

**Java:** `HashMap<String, int[]>` (or `List<Integer>`) for postings, `Locale.ROOT` for lower-casing, and a `BitSet` or `ConcurrentHashMap.newKeySet()` for deleted ids; `Character.isLetterOrDigit` decides the word boundaries. A `ReentrantReadWriteLock` or a volatile snapshot keeps readers from blocking each other.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

# Hints — 115 Autocomplete / Typeahead Service (Rust)

Try the problem for at least 25 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

For the prefix `ap`, which terms do you ever need to look at? What do all the strings you may return have in common, and could the data be laid out so that the work for a query depends on the length of the prefix and on `k`, not on how many terms exist?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

A prefix tree (trie) gives you "all terms that share this prefix" as one place to start; a sorted map with range scans is the alternative. To avoid walking the whole subtree on each query, each node can remember its best few terms, which moves work from reads to writes, so think about what has to be updated when one count changes. Keep the ranking rule behind one small abstraction (what it compares, not how it is stored) so recency, frequency and a per-user score are interchangeable, and keep per-user counts in their own structure instead of copying the tree.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `Term { text, count (atomic), lastRecordedAt }` owned once; an index from prefix to terms (trie nodes holding, for each node, an immutable ranked list of at most K terms, or a sorted map you range-scan with a bounded heap); `Ranking` = a comparator over `Term` plus the final alphabetical tie-break; `UserHistory` = `user -> term -> count` consulted only by the personalised query, which takes the global candidates (take more than `k` of them) and re-orders them. On `record`: validate and normalise first, bump the term's atomic count and timestamp, then fix the ranked lists of the nodes on the path (insert, move or leave alone), publishing each new list in one step so readers never see a half-sorted one.

**Rust:** `RwLock<BTreeMap<..>>` is the straightforward start; per-entry atomics (`AtomicU64`) or sharding the map by prefix lets writers stop blocking each other. Do not use `DefaultHasher` for anything that must be stable across runs. Normalise with `str::to_lowercase`.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

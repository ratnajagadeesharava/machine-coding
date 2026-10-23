# Hints — 116 In-Memory Search Engine (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

A search for a rare word should not have to look at every document. What do you have to remember at add time so that, given a word, you can find the documents that contain it? And what does a delete or update therefore have to undo?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Keep two views of the same data: per document, its term counts and length; per term, the set of documents containing it. The score needs `N` and `df`, and both are just sizes of these structures at the moment of the query, so nothing needs to be stored per score. Separate three jobs in your head: turning text into terms, finding candidate documents (AND = intersect, OR = union), and scoring/ordering them. The tokeniser and the scorer are the two parts most likely to change, so give each one a name and a single responsibility. A query with several options (terms, mode, paging) reads better when it is built step by step than when it is a long parameter list.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `Index { docs: id -> {term counts, token count}, postings: term -> set of ids }`; `Tokeniser` (text to terms) and `Scorer` (index view + terms + doc to score) as small replaceable parts; `search` = tokenise, de-duplicate terms in first-seen order, pick candidates from postings (start the AND intersection from the smallest posting list), score each candidate, sort by (score desc, id asc), slice by offset/limit. Update = delete then add under the same exclusive section, so nobody sees the gap. Guard the whole index with a read/write lock (many searches, rare writes), or publish immutable snapshots.

**Rust:** `RwLock<Index>` with `HashMap<String, BTreeSet<String>>` for postings; compare `f64` scores with `total_cmp` (or `partial_cmp` after checking no NaN) when sorting, and build the result `Vec<String>` while the read guard is held.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

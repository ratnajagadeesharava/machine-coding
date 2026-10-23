# Hints — 119 Mini RAG Retrieval Component (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

List the verbs the component has (cut text, embed, store, search, assemble). For each one, ask what could change independently of the others tomorrow (a different model, a different database, a different prompt wording, a different way to cut text) and what the rest of the code needs to know about it.

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Give each part that may be swapped its own small seam: chunking, embedding and vector storage each get one narrow interface, and the engine only orchestrates them (the tests already provide two of the three seams, so mirror that style for the third). Do the slow work (cutting and embedding a document) before you take any lock, then apply the result to the shared state in one short critical section. Keep the per-document list of chunk ids so that replacing or deleting a document never has to scan the whole store or match on id prefixes.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `Chunker` (text, size, overlap -> list of strings) -> `Embedder` (provided) -> `VectorStore` (provided interface, plus your own in-memory implementation that scans all entries and keeps the k best with a bounded priority queue) with a `docId -> chunk ids` map owned by the engine. `ingest` = chunk + embed outside the lock, then under a write lock remove the old chunk ids from the store and the map, upsert the new ones, and record the new ids. `retrieve` = embed the query, search the store under a read lock, map hits to ids. `buildPrompt` = the same hits, then a small `PromptBuilder` that walks them in order, tracks the running length including the separators and the `[n] ` prefix, and stops at the first entry that would go over.

For the cosine score keep each vector's norm next to it (computed once at `upsert`) and treat a zero norm as score 0; sort by `(score desc, id asc)` and compare scores exactly so equal chunks tie. **Java:** `ReentrantReadWriteLock` (or a copy-on-write snapshot of the store) fits a read-mostly workload; `String::compareTo` is the id order the tests expect.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

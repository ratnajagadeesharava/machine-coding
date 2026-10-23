# Interviewer notes — 119 Mini RAG Retrieval Component

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- A slow embedding of one document (embedder blocked inside `embed`) must not block ingest, delete, retrieve or prompts for other documents: the candidate must embed outside the engine's lock.
- Re-ingesting a document while readers query it: readers see 4 chunks or 2 chunks of the same version, never 6 or a prompt that mixes old and new text. Also the "first chunk replaced, second not yet" window.
- Delete of `a` must not touch `ab#0` or `a#1#0` (prefix matching on ids is the classic bug).
- A failing embedder on the third chunk of a brand-new document: no chunk of it may be stored (the supplied store must stay empty). A candidate who upserts chunk by chunk leaves a partial document behind.
- Query that matches nothing: all scores 0, result ordered by chunk id; a zero-norm vector must not produce NaN (0/0) that breaks sorting.
- Ties across `n#10` and `n#2`: string (byte) order, not numeric order. A candidate who stores chunks in a list and sorts stably by score returns insertion order instead.
- 100,000-word document: 1,250 chunks with the default 100/20 chunking; quadratic string building or recursion over words will show up.
- Re-ingesting identical text twice leaves identical state (ids, store contents, results).
- Large corpus (4,000 chunks, 60 queries): ranking must equal a brute-force scan (scores compared with a tolerance, so equal-score ties may differ only if the candidate's arithmetic differs).

## Likely interviewer questions
- Why is the embedder called outside the lock, and what can go wrong with two concurrent ingests of the same document id? (Last committed write wins; both are consistent; ids never mix.)
- What does `delete_document("a")` do, and how do you know which chunks belong to `a`? (A doc -> ids map, not prefix scanning.)
- What happens to `set_chunking` for documents already stored? Would you store the chunking parameters per document?
- Where does the budget rule live, and why stop at the first chunk that does not fit instead of skipping it? (Prompt is a ranked prefix: never include a worse chunk instead of a better one.)
- Why return `(none)` instead of throwing when nothing fits?

## Invariants
- The vector store holds exactly the union of the current chunks of all documents; the engine's own map and the store never disagree after any call returns, including failed ones.
- A chunk id is `doc_id#n` with n contiguous from 0 per document.
- A reader sees every document either before or after a replace, never in between.
- Ranking is a total order: score descending, then chunk id ascending.
- The embedder is called exactly once per chunk and once per query.

## Common mistakes
- Replace implemented as delete-then-insert in two lock acquisitions (readers see the gap) or as insert-then-delete by id prefix (deletes the new chunks).
- Leaving stale chunks when the new version has fewer chunks than the old one.
- Embedding inside a global lock, so one slow call stalls every reader.
- Off-by-one chunking: a redundant trailing chunk that is a subset of the previous one; infinite loop when `overlap >= chunk_size` is not validated; a final chunk that drops the tail words.
- Not validating `set_chunking` before mutating (half-applied settings).
- Cosine without a zero-norm guard (NaN), or comparing NaN scores in a comparator (`partial_cmp(..).unwrap()` panic).
- Counting only chunk text in the budget and forgetting the `[n] ` prefix and the `\n` separators; or checking the budget after appending.
- Skipping a too-large chunk and including a smaller lower-ranked one.
- Using `HashMap` iteration order for ties (Rust randomises it per process).
- Holding the engine lock across calls into a store that has its own lock in a way that can deadlock.

## Over-engineering traps
- An approximate nearest-neighbour index (HNSW, IVF) before the exact scan works and is tested.
- A plug-in registry, dependency injection container or event bus for three collaborators.
- Making the prompt template configurable through a template language when one exact template is specified.
- Persisting chunks to disk, sharding the store or adding an async pipeline in the core.

## Design patterns and structures to listen for
- **Strategy** for each swappable part: `Chunker` (fixed-window now, sentence-aware or token-based later), `Embedder` and `VectorStore` (both provided as interfaces; the candidate should keep depending on the interface and not on the test fakes), and optionally a `PromptTemplate` / `ContextPacker` for the budget rule.
- A small **pipeline / facade**: `RagEngine` orchestrates chunk -> embed -> store and query -> embed -> search -> pack; it holds no scoring or text-cutting logic itself.
- **Value objects**: `Chunk(id, doc_id, index, text)`, `Hit`.
- Data structures: `Map<doc_id, List<chunkId>>` for replace/delete; a bounded min-heap of size k for top-k instead of sorting the whole store (O(n log k)); precomputed vector norms; an `RwLock` or an `Arc` snapshot swapped on commit for the read-mostly state.
- A **builder** or a small result object for the prompt is fine but a single `String.format` with the exact template is also acceptable. Do not reward template engines.

## Design and scaling discussion
- 10M chunks: exact scan is O(n·d) per query; discuss ANN indexes (HNSW graph, IVF clusters, product quantization), the recall/latency trade-off, memory (10M x 768 floats is about 30 GB; quantize to 8 bits or shard), and why the store interface (`upsert`, `remove`, `top_k`) is already the right seam for swapping in such an index.
- Deleting from ANN indexes is usually a tombstone plus periodic rebuild; how does that interact with "exactly the current chunks"?
- Embedding is the expensive and failure-prone step: batching, retries, timeouts, caching by text hash, and why a failed embed must not leave partial state.
- Chunking quality drives answer quality: overlap preserves sentences cut at a boundary; chunk size trades precision for context; token-based sizes match the model's window better than words.
- Prompt budget in characters is a proxy for tokens; discuss a tokenizer-aware budget and the order of context (most relevant first or last).
- Consistency: single engine lock vs per-document versions; what a reader should see during a streaming ingest (V5).

## Trade-off questions to ask when the candidate is done
- Two concurrent ingests of the same document with different text: what are the possible final states, and are they acceptable?
- Would you keep chunk text in the engine, in the store payload, or both? What does each cost when the store is remote?
- If the store were remote and slow, which of your locks would you be holding during its calls?
- How would you test that the engine does not hold its lock while embedding, without sleeping?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.

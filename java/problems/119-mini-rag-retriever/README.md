# 119 · Mini RAG Retrieval Component (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 2 — the collaborator interfaces (embedder, vector store) and the tests are provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

A language model only knows what it was trained on, so a question about your own documents needs the relevant passages pasted into the prompt. This is retrieval-augmented generation (RAG). You are building the retrieval half: documents are cut into overlapping **chunks**, every chunk is turned into a vector by an **embedder**, and a question is answered by finding the chunks whose vectors are most similar to the question's vector and assembling them, within a size budget, into a **prompt**. There is no real model and no network: the embedder is a collaborator that the tests implement with deterministic fakes, and so is an alternative **vector store**.

## Problem

Build a thread-safe component that ingests documents, retrieves the chunks most relevant to a question, and builds the prompt a language model would receive. Documents come and go (re-ingesting a document replaces it, documents can be deleted) while questions are being asked. The component must work with whatever embedder it is given, and with either its own built-in vector store or one handed in by the caller.

## Requirements

1. **Chunking.** The text is split on whitespace into words. A chunk holds at most `chunkSize` words; consecutive chunks of one document share `overlap` words (the last `overlap` words of a chunk are the first `overlap` words of the next). Chunk `n` starts at word `n * (chunkSize - overlap)`. Chunks are produced until one reaches the last word; no extra chunk is created that is entirely contained in the previous one. The text of a chunk is its words joined by single spaces (original whitespace is not kept).
2. Defaults are `chunkSize = 100`, `overlap = 20`. `setChunking(chunkSize, overlap)` changes them for later ingests only; chunks already stored are untouched. `chunkSize >= 1` and `0 <= overlap < chunkSize` are required, otherwise `IllegalArgumentException` and the old setting stays.
3. **Ingest.** `ingest(docId, text)` creates the chunks, embeds each chunk with the embedder, stores them under the ids `docId#0`, `docId#1`, ... (written `docId + "#" + n`, n from 0) and returns the number of chunks. Empty or blank text creates 0 chunks.
4. Re-ingesting a `docId` replaces all its previous chunks. Readers see the document either entirely as it was or entirely as it is now, never a mixture. If the embedder throws while ingesting, the exception propagates and nothing changes (the previous version of the document, if any, stays).
5. **Delete.** `deleteDocument(docId)` removes exactly the chunks of that document and returns whether the document was known. Other documents are not affected, even if their ids start with the same characters.
6. **Retrieve.** `retrieve(query, k)` embeds the query and returns the ids of the `min(k, chunkCount())` chunks with the highest cosine similarity to it, best first. Equal scores are ordered by chunk id ascending (plain `String.compareTo` order, so `d#10` comes before `d#2`). A zero vector has similarity 0 with everything. Chunks with similarity 0 are still returned if there is room.
7. **Prompt.** `buildPrompt(query, k, maxContextChars)` uses the chunks `retrieve(query, k)` would return, in that order, and produces exactly this text (`\n` is a line feed, there is no trailing newline):

   ```
   Answer using only the context.\n
   \n
   Context:\n
   <context>\n
   \n
   Question: <query>\n
   Answer:
   ```

   `<context>` is the entries `[1] <chunk text>`, `[2] <chunk text>`, ... joined by `\n`, numbered from 1 in rank order. `<query>` is the query exactly as given. If no entry is included, `<context>` is the text `(none)`.
8. **Budget.** The length of `<context>` (all included entries and the `\n` separators between them) must not exceed `maxContextChars`. Entries are added in rank order; whole chunks only, a chunk is never cut. At the first chunk that would make the context too long, stop: later (smaller) chunks are not considered. If even the first chunk does not fit, or `maxContextChars` is 0, the context is `(none)`.
9. The embedder is called once per chunk at ingest and once per query in `retrieve` and `buildPrompt`; nothing else calls it. Vectors of one embedder all have the same length.
10. Collaborators: the component can be created with an embedder alone (it then uses a vector store of its own) or with an embedder and a caller-supplied vector store. Everything stored there (add, replace, remove) must be kept in step with the document operations, so that the supplied store holds exactly the chunks that are currently ingested.
11. **Errors.** `IllegalArgumentException` for: a null/blank `docId`, null `text`, `k < 1`, a null/blank `query`, a negative `maxContextChars`, a null embedder or a null store passed to `create`.
12. **Concurrency.** All operations may be called from many threads at once. Ingests and deletes of different documents must not lose each other's work, and a slow embedding of one document must not stop other documents from being ingested, deleted or queried.

## Examples

```
setChunking(4, 1);  ingest("doc", "w0 w1 w2 w3 w4 w5 w6 w7 w8 w9")  -> 3
chunkText("doc#0") = "w0 w1 w2 w3"   chunkText("doc#1") = "w3 w4 w5 w6"   chunkText("doc#2") = "w6 w7 w8 w9"

ingest("a", "alpha beta");  ingest("b", "alpha gamma delta");  ingest("c", "alpha")      (default chunking: one chunk each)
retrieve("alpha", 2)  -> [c#0, a#0]          (cosine 1.0, then 0.71; b#0 would be third with 0.58)

buildPrompt("alpha", 3, 1000) ->
  "Answer using only the context.\n\nContext:\n[1] alpha\n[2] alpha beta\n[3] alpha gamma delta\n\nQuestion: alpha\nAnswer:"

buildPrompt("alpha", 3, 24)   -> context is "[1] alpha\n[2] alpha beta" (9 + 1 + 14 = 24 characters)
buildPrompt("alpha", 3, 23)   -> context is "[1] alpha"
buildPrompt("alpha", 3, 8)    -> context is "(none)"
```

### Worked traces

```
setChunking(2, 0)
ingest("doc", "alpha beta gamma delta epsilon zeta") -> 3     ingest("other", "eta theta") -> 1      chunkCount() = 4
ingest("doc", "iota kappa")  -> 1       chunkCount() = 2      chunkText("doc#1") = empty
ingest("doc", "")            -> 0       retrieve("alpha", 10) -> [other#0]    (the document is gone)

ingest("b","alpha"); ingest("a","alpha"); ingest("c","alpha")
retrieve("alpha", 3) -> [a#0, b#0, c#0]                       (equal scores: by chunk id)
one document "n" of 12 chunks all equal to "alpha"
retrieve("alpha", 12) -> [n#0, n#1, n#10, n#11, n#2, ..., n#9] (string order)

ingest("c","alpha"); ingest("big","alpha alpha alpha beta"); ingest("small","alpha beta beta")
retrieve("alpha", 3) -> [c#0, big#0, small#0]        entries have 9, 26 and 19 characters
buildPrompt("alpha", 3, 30) -> context "[1] alpha"   ("[2] ..." would make 36 > 30; "[3] ..." is not tried although 9 + 1 + 19 = 29 fits)
```

### Edge cases

- Text with no words (empty, spaces, newlines) creates 0 chunks and, when it re-ingests a known document, removes that document.
- Last chunk: 7 words with `(4, 1)` give 2 chunks (words 0-3 and 3-6); the chunk starting at word 6 would be a subset of the second one and is not created. 4 words with `(4, 1)` give 1 chunk.
- `retrieve` with fewer chunks than `k` returns all of them; with no chunks it returns an empty list. `buildPrompt` with no chunks has context `(none)`.
- A query that shares nothing with any chunk (zero vector or zero similarity everywhere) returns chunks ordered by chunk id.
- Numbering costs characters: `[10] ` is one character longer than `[9] `, and that counts against the budget.
- Doc ids may contain `#` or be prefixes of each other (`a`, `ab`, `a#1`); deleting `a` removes only the chunks `a#0`, `a#1`, ... created from `a`.
- Chunking changes apply to later ingests only. Documents ingested earlier keep their chunks until they are re-ingested.
- All sizes are counted in characters of the text (the tests use ASCII only).

## Glossary

- **Chunk**: a run of consecutive words of a document; the unit that is embedded, stored and retrieved.
- **Overlap**: the number of words that two consecutive chunks of one document share.
- **Embedder**: turns a piece of text into a vector of numbers (provided by the tests).
- **Cosine similarity**: `dot(a, b) / (|a| * |b|)`; 1 means the same direction, 0 unrelated; defined as 0 if either vector is all zeros.
- **Vector store**: keeps vectors with an id and a payload and finds the nearest ones to a query vector (provided by the tests as an alternative; you also need your own).
- **Top-k**: the k best-scoring items.
- **Context budget**: the maximum number of characters of retrieved text that may go into the prompt.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `IllegalArgumentException` as listed in requirement 11. An embedder exception is not wrapped; it propagates unchanged from `ingest`, `retrieve` and `buildPrompt`.
- Rust: `Result<_, RagError>` with `InvalidArgument` and `Embedding(String)` (the embedder failed); `k`, sizes and budgets are `usize`, so only `k = 0` and invalid chunking need checking.
- The prompt template, the `(none)` text, the `[n] ` numbering and the budget rule are exact.
- Chunk ids are `docId#n`; `chunkText` returns the stored text of a chunk (words joined by single spaces).
- `VectorStore.topK` returns hits ordered by score descending and id ascending, and each hit carries the payload given to `upsert`.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. `Embedder` and `VectorStore` are implemented by the tests; create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

/** Turns text into a vector. Implemented by the tests. All vectors from one embedder have the same length. */
public interface Embedder {
    double[] embed(String text);
}

/** Implemented by the tests as an alternative store; thread-safe. */
public interface VectorStore {
    record Hit(String id, double score, String payload) {}

    /** Inserts, or replaces the vector and payload stored under id. */
    void upsert(String id, double[] vector, String payload);

    /** Removes the entry; true if it existed. */
    boolean remove(String id);

    /** At most k hits (k >= 1), best cosine similarity first, equal scores by id ascending. Zero vectors score 0. */
    List<Hit> topK(double[] query, int k);
}

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface RagEngine {
    int ingest(String docId, String text);
    void setChunking(int chunkSize, int overlap);
    boolean deleteDocument(String docId);
    List<String> retrieve(String query, int k);
    String buildPrompt(String query, int k, int maxContextChars);
    int chunkCount();
    Optional<String> chunkText(String chunkId);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static RagEngine create(Embedder embedder)                      // your own default vector store
    public static RagEngine create(Embedder embedder, VectorStore store)   // the given vector store
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. The fakes are in `src/test/java/playground/Fakes.java`: an embedder that counts occurrences of 16 known words, a 26-letter-histogram embedder, and a brute-force vector store that records how it was used. Nothing depends on time, randomness or sleeping.

- Chunking: size, overlap, the last chunk, blank text, whitespace normalisation, defaults, validation, later-ingest-only.
- Ingest/replace/delete bookkeeping: ids, counts, replacement, deletion, ingest after delete.
- Ranking: cosine order, length independence, tie-breaking (including `d#10` vs `d#2`), k bounds, zero-similarity chunks, empty index, bad arguments.
- Prompt: exact template, budget boundaries, `(none)`, stopping at the first chunk that does not fit, multi-digit ranks, verbatim query.
- Swappability: a second embedder, a supplied store (kept in step with the documents), identical answers with the default and a supplied store, embedder call counts, embedder failure leaving the old version.
- Concurrency: 8 threads ingesting and querying; 8 threads ingesting and deleting.
- More edge cases (slow embedding of one document, half-replaced documents, prefix ids, large corpus, failed first ingest) are in `interviewer/`.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/119-mini-rag-retriever test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`Candidate`, `Concurrently`): `java/testkit`.

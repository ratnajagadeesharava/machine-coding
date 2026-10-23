# 116 · In-Memory Search Engine (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

A small knowledge-base tool keeps thousands of short documents in memory and lets people find them by typing a few words. You are building the search core: documents are added with an id and some text, the text is split into **terms**, and a query returns the documents that contain all (or any) of the query's terms, best match first. "Best" is decided by a fixed relevance score (**TF-IDF**) that rewards documents where the query terms are frequent and penalises terms that appear in almost every document. Documents can be deleted or rewritten at any time, the very next search must reflect it, and many threads search while others edit.

## Problem

Build the search engine described above.

- A document is an id and a text. The text is turned into terms by a fixed **tokeniser**.
- A query is a text too; it goes through the same tokeniser.
- A search runs in one of two modes, **AND** (a document must contain every query term) or **OR** (a document must contain at least one query term), and returns the ids of the best `k` documents, or one page of the ranking.
- The ranking is a TF-IDF score, defined exactly below so that every score and every order is reproducible.

## Requirements

1. **Tokeniser.** Lower-case the text, split it on every character that is not a letter or a digit, and drop empty pieces. No stop words are removed. `"Hello, WORLD! foo_bar-2024"` gives `hello`, `world`, `foo`, `bar`, `2024`. A text may have no token at all (for example `""` or `"?!"`); such a document is valid, is never matched, and still counts as a live document.
2. `add_document``(docId, text)` adds a document. Adding an id that is currently live is an error. A deleted id may be added again.
3. `updateDocument(docId, text)` replaces the text of a live document; `deleteDocument(docId)` removes a live document. Both are errors for an id that is not live. The effect is visible to the very next search, score and count.
4. **Query terms.** The query text goes through the tokeniser; repeated terms count once (`"fox fox"` is the same as `"fox"`). A query without any term matches nothing, in both modes.
5. **Matching.** AND: the document contains all the (distinct) query terms. OR: it contains at least one. A query term that no live document contains matches nothing: it makes an AND query return nothing and adds nothing to an OR query.
6. **Score.** With `N` = number of live documents (including documents without tokens), and for a term `t`: `df(t)` = number of live documents containing `t`; `tf(t, d)` = number of occurrences of `t` in `d` divided by the number of tokens of `d`; `idf(t) = ln(1 + N / df(t))` (natural logarithm, `N / df` computed in floating point). `score(d) = Σ tf(t, d) · idf(t)` over the distinct query terms `t` that occur in `d`, added in the order of their first appearance in the query. Everything is recomputed from the live documents at query time: deleting, updating or adding documents changes the `N` and `df` of later searches.
7. **Ranking.** Matching documents are ordered by score, highest first; equal scores are ordered by document id ascending (plain string comparison). Scores of documents that are mathematically equal (for example `1/2` and `2/4` term frequencies) must be treated as equal.
8. `search(query, mode, k)` returns the ids of the first `k` documents of the ranking (fewer if there are fewer matches; `k = 0` gives an empty list). `search(query, mode, offset, limit)` returns the ids ranked `offset` to `offset + limit - 1`; an offset beyond the end gives an empty list. Paging through a ranking in consecutive pages gives the same list as one big search (when nothing changes in between).
9. `scoreOf(query, docId)` returns the score of a live document for the query as defined above, whatever the mode; `0.0` when no query term occurs in it. It is an error for an id that is not live.
10. `documentCount()` returns the number of live documents.
11. Invalid input is an error and changes nothing (see the behavioral contract).

## Constraints

- The engine is used from many threads at once: searches run while documents are being added, updated and deleted. It must be thread-safe.
- A search sees the collection either before or after any single add, update or delete, never a mixture (an updated document is never found by terms from both its old and its new text in an AND query that needs both).
- Two threads adding the same new id at the same time: exactly one succeeds, the others get the duplicate error. No document may be lost when 16 threads add documents concurrently.
- Searches must not scan every document when the query terms are rare (hidden: 30,000 documents, 20,000 selective queries within seconds).
- Everything is deterministic: no randomness, no clock.

## Examples

```
add_document("d1", "the quick brown fox")
add_document("d2", "the lazy dog")
add_document("d3", "quick quick fox")          N = 3

search("quick fox", "OR", 10)    -> Ok(["d3", "d1"])
score_of("quick fox", "d3")      -> Ok(0.9162907318741551)        (tf 2/3 + 1/3 = 1, idf ln(1 + 3/2))
score_of("quick fox", "d1")      -> Ok(0.45814536593707755)       (tf 1/4 + 1/4 = 1/2)
search("lazy fox", "AND", 10)    -> Ok([])                        (no document has both)
search("lazy fox", "OR", 10)     -> Ok(["d2", "d3", "d1"])        (lazy is rare: 1/3 * ln(1 + 3/1) = 0.4621 beats 0.3054, 0.2291)
```

```
delete_document("d3")                                  N = 2
score_of("fox", "d1")  -> Ok(0.25 * ln(1 + 2/1)) = 0.27465307216702745      (it was 0.25 * ln(1 + 3/2) = 0.2291 before)

update_document("d2", "quick quick quick dog")         N = 2
search("quick", "OR", 10)  -> Ok(["d2", "d1"])         (tf 3/4 vs 1/4, both df = 2; "lazy" no longer matches anything)

add_document("d1", "again")  -> Err(DuplicateDocument)      search("fox", "XOR", 5) -> Err(InvalidArgument)
```

```
add_document("b", "red blue"); add_document("a", "blue red"); add_document("c", "red blue green green")
search("red", "OR", 10)           -> Ok(["a", "b", "c"])     (a and b have the same score exactly; doc id ascending; c has the lowest tf)
search_page("red", "OR", 1, 1)    -> Ok(["b"])               (offset 1, limit 1)
search_page("red", "OR", 5, 10)   -> Ok([])                  (offset past the end)
```

### Edge cases

- Tokenisation is on non-alphanumerics, so `foo_bar` is two terms (`foo`, `bar`) and `x100x200` is one term. Case never matters: `WoRlD` finds `world`.
- Term frequency is relative to the length of the document: the same word once in a 2-token and once in a 10-token document gives `1/2` and `1/10`.
- Documents with no tokens count in `N` (they lower the idf of every term) but never appear in results.
- When the last live document is deleted `N = 0`; all searches return nothing, and adding a document again starts from scratch.
- Query `"fox zebra"` in OR mode ranks and scores exactly like `"fox"` if no document contains `zebra`; in AND mode it returns nothing.
- `k = 0` or `limit = 0` returns an empty list (not an error); a negative value is an error where the type allows it.
- Mode names are case-sensitive: `"and"` is an error.
- A document id is used exactly as given (no trimming, case-sensitive); only a null/empty/whitespace-only id is invalid.

## Glossary

- **Term / token**: a lower-cased run of letters and digits produced by the tokeniser.
- **Tokeniser**: the fixed rule turning text into tokens (requirement 1).
- **Live document**: added and not (yet) deleted.
- **tf**: how frequent a term is inside one document, relative to that document's length.
- **df**: in how many live documents a term occurs at least once.
- **idf**: how rare a term is across the collection; rare terms weigh more.
- **Ranking**: all matching documents ordered best first; **page**: a slice of it given by offset and limit.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Rust: `Result<_, SearchError>` with `InvalidArgument` (blank `doc_id`, `mode` other than exactly `"AND"` or `"OR"`), `DuplicateDocument` (`add_document` of a live id), `UnknownDocument` (`update_document` / `delete_document` / `score_of` of an id that is not live). `k`, `offset` and `limit` are `usize`, so they cannot be negative. Failed calls leave the engine unchanged.
- The mode is the exact string `"AND"` or `"OR"`.
- The scores returned by `scoreOf` are compared with the formula above to within `1e-9`; the order of results must be exactly the order given by the formula and the tie rule.
- Letters and digits in the tests are ASCII; behaviour for other scripts is yours to decide.
- A failed add / update / delete leaves the previous state untouched.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below. It is **not** your domain model — it is the doorway. Create whatever you like and wire it up.

```rust
pub enum SearchError { InvalidArgument, DuplicateDocument, UnknownDocument }

pub trait SearchEngine: Send + Sync {
    fn add_document(&self, doc_id: &str, text: &str) -> Result<(), SearchError>;
    fn update_document(&self, doc_id: &str, text: &str) -> Result<(), SearchError>;
    fn delete_document(&self, doc_id: &str) -> Result<(), SearchError>;
    fn search(&self, query: &str, mode: &str, k: usize) -> Result<Vec<String>, SearchError>;
    fn search_page(&self, query: &str, mode: &str, offset: usize, limit: usize) -> Result<Vec<String>, SearchError>;
    fn score_of(&self, query: &str, doc_id: &str) -> Result<f64, SearchError>;
    fn document_count(&self) -> usize;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create() -> Option<Arc<dyn SearchEngine>>
```

## What the tests check

These tests are black-box: they only call the contract above, so any design behind it works. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Tokenisation (case, punctuation, underscores, digits), the exact TF-IDF values on a small corpus, length normalisation.
- AND vs OR, unknown terms in both modes, tie-break by document id, `k`, and pagination (pages glue back to the full ranking, offsets past the end, zero limit).
- Duplicate ids, invalid arguments, empty and repeated query terms, documents without tokens.
- Delete, update and re-add: results, idf and counts change immediately; unknown ids are rejected.
- Concurrency: 16 threads adding 200 documents each; 12 threads racing to add the same id; searches running while documents are added and deleted.
- Harder scenarios (a brute-force oracle over random add/update/delete operations, update atomicity under concurrent searches, performance on 30,000 documents, paging through a shrinking collection) are in `interviewer/`.

## Running

```bash
cargo test                       # from this directory
STRICT=1 cargo test              # fail instead of skip while unimplemented
cargo test --features hidden     # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation print `SKIPPED` and return until `create` is wired up, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`sut!`, `run_concurrently`): `rust/support`.

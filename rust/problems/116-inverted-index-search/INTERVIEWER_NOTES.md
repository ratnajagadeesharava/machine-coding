# Interviewer notes — 116 In-Memory Search Engine

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- idf depends on `N`, which includes documents without tokens and documents that match nothing: adding unrelated documents changes scores.
- Update must change tf, df and N effects at once: after `update`, old-only terms must have no posting left and the new ones must be visible; a stale posting makes AND/OR return a document that no longer contains the term.
- Delete of the last document, then add again: empty collection, `N = 0`, no division by zero anywhere (a document with 0 tokens must not divide by its length).
- Repeated query terms counted twice (`"fox fox"` doubles the score) or an AND query that intersects the same list twice with a wrong result.
- Tie handling: `1/2` and `2/4` must tie exactly (compute tf as a division of the two integers, not an accumulated float), and the final order is by doc id, not insertion order or hash order.
- Unknown term in an AND query must short-circuit to empty; in OR it must not throw or contribute.
- Pagination: the page boundaries must come from one sorted ranking, not from sorting each page separately; offset past the end is empty, not an error.
- A reader must never observe a half-applied update (old terms removed, new ones not yet added), see `should_never_show_a_half_applied_update_to_concurrent_searches`.
- Two concurrent adds of the same id: check-then-act on a plain map loses the race.
- Performance: a scan over all documents per query passes the small tests and fails the 30,000-document one.

## Likely interviewer questions
- Walk me through the data structures. What does `add`, `delete`, `search` cost?
- Why is idf not stored per term? What would you have to maintain if you did?
- How do you intersect posting lists for an AND query? Which list do you start from, and why?
- How do you get the top 10 out of 1,000,000 matches without sorting all of them?
- What does a search see while another thread updates a document? How did you achieve that, and what does it cost the writers?
- Where would you plug in a stemmer or a different tokeniser? A BM25 scorer? What has to change in the index when you do?

## Invariants
- Every (term, doc) pair in a doc's term counts has exactly one matching entry in that term's postings, and vice versa; empty postings are removed.
- `documentCount` equals the number of entries in the document table.
- `df(t)` equals the size of `t`'s postings at the time of the query; `N` is the size of the document table.
- Every search sees a state that existed between two complete operations.

## Common mistakes
- Storing `tf * idf` at index time (goes stale on every add/delete).
- Computing tf as count / total tokens of the corpus, or forgetting to divide by the document length.
- `N` counted as documents containing at least one query term.
- Forgetting to remove a document's postings on update, or removing them but not the term counts.
- Sorting with a comparator that subtracts doubles and casts to int; comparing doubles with `==` after summing in different orders.
- Locking per structure (documents map and postings map separately) so a reader sees one updated and the other not.
- `String.split("[^a-z0-9]")` on the original (not lower-cased) text, losing capital letters; or `split` leaving a leading empty token.
- Returning a live view of an internal collection from `search`.

## Design patterns and data structures an interviewer hopes to hear
- **Strategy** for the tokeniser (lower-case/alphanumeric today; stemming, n-grams, stop words, language-specific later) and for the scorer (TF-IDF today, BM25 tomorrow). The index must not know which is plugged in; the scorer needs a read-only view of the statistics (`N`, `df`, tf, doc length).
- **Builder** for the query: terms, mode, offset/limit (later: phrase, fields, boosts, fuzzy) are optional and grow; a fluent builder or a small immutable query object avoids overload explosions like `search(q, mode, offset, limit, boosts, ...)`.
- **Inverted index**: term to postings (set, sorted list or map doc to tf). Mention positional postings for phrase search, skip lists / sorted-ID merging for intersections, and a top-k heap instead of a full sort.
- **Concurrency**: read/write lock (or copy-on-write snapshot / segment-based design like Lucene's immutable segments plus a deletion bitmap) so searches are not blocked by each other; update as one critical section (delete + add).
- Possible additional seams: a `DocumentStore` (id to text and counts) separate from the `PostingsIndex`, so storage can change.

## Over-engineering traps
- An abstract factory of tokenisers, a plugin loader and a query-language parser before AND/OR works.
- A persistent on-disk index, compression of postings, or a distributed design in the core.
- Caching scores per (query, doc) when idf changes on every write.

## Design and scaling discussion
- Memory: postings dominate; mention doc-id interning (int ids), delta encoding and variable-byte compression.
- Top-k: a bounded min-heap makes ranking `O(m log k)` instead of `O(m log m)` for `m` matches; deep pagination (large offsets) still costs `O(m)`; discuss search-after cursors.
- Exact idf needs global counts; with sharding, either gather shard statistics first (two-phase query) or accept per-shard idf and its ranking skew.
- Near-real-time: write to a small in-memory segment, search the union of segments, merge in the background; deletes are tombstones filtered at query time.
- Scoring alternatives: BM25 saturates tf and normalises length better than the plain ratio used here; discuss why TF-IDF over-rewards repeated words in short documents.

## Trade-off questions to ask when the candidate is done
- One global lock vs a lock per term: what happens to an update touching 50 terms and a query touching 3?
- Snapshot per operation (copy-on-write) vs RW lock: write amplification vs reader latency.
- Do you validate/normalise the doc id? What about two ids differing only by case?
- If tf were stored per posting (doc to count) instead of per document, what gets faster and what gets harder?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.

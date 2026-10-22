# Interviewer notes — 109 Document Line Search

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- A blank or punctuation-only query must return an empty list; a naive "all terms present" loop over zero terms returns every line.
- A query word that appears nowhere must give an empty result even when the other words are common.
- Ids after deletions: deleting line 1 must not shift the others; deleting the last line must not allow its id to be handed out again.
- Racing deleters: with 16 threads deleting the same id, exactly one gets `true`.
- A search that overlaps deletions must be sorted and must never contain an id whose deletion had already returned.
- Rare word plus common word on a large document: results must still be in id order when the intersection starts from the smallest list.
- Non-ASCII case folding (`ÉCOLE` vs `école`), newline/tab inside a line as separators, and a very long line.
- Mutating the returned list, or the list passed to `create`, must not change the document.
- 100,000 lines and thousands of searches: a per-search scan of every line, even over pre-split words, is too slow once the document grows.

## Likely interviewer questions
- What is your definition of a word, and where is it implemented? Is the query split by the same code?
- What is the cost of one search? Of building the index? Of a delete?
- How do you intersect the posting lists, and which list do you start from?
- Why not remove the id from the index when a line is deleted? What do you pay for filtering at query time?
- How would you handle `toLowerCase` in a Turkish locale? What about characters that change length when lowercased?
- What do you return for an empty query, and why is that the right call for a search box?

## Invariants
- Ids are positions at load time; they are never renumbered or reused.
- Every posting list is sorted ascending and has no duplicates, so intersections and the final result are sorted.
- A deleted id is never returned; a deletion becomes visible to searches atomically.
- The same tokenizer produces the words of lines and of queries.

## Common mistakes
- Using `String.contains` / `indexOf` on the line, so `cat` matches `category`.
- Splitting on whitespace only, so `cat,` and `cat.` do not match `cat`.
- Forgetting to lower-case one of the sides, or lower-casing with the default locale.
- Re-tokenizing every line on every search (passes the small tests, fails the 20,000-line time bound).
- Returning all lines for an empty query, or throwing for it.
- Posting lists as lists with duplicates (a line that repeats a word appears twice in the result).
- Reusing `list.size() + 1` as the next id after deletions, or removing from a list so ids shift.
- Returning the internal list, so a caller's `remove` changes the index.
- Checking "is it live" and deleting in two steps without atomicity, so two threads both get `true`.
- Taking one exclusive lock for searches, serialising all readers.

## Over-engineering traps
- A full query parser (boolean operators, quotes, wildcards) before AND-of-words works.
- A trie, suffix array or n-gram index when whole-word matching needs a map from word to ids.
- A plugin architecture for tokenizers and analyzers; one function is enough for this problem.
- Persisting the index to disk as part of the core.

## Design and scaling discussion
- Inverted index: `word -> sorted ids`. Intersect starting from the shortest list; with skewed lists, probing the long list by binary search beats a linear merge.
- Deletes: a tombstone set/bitset checked at query time is O(1) per candidate and keeps the posting lists immutable (lock-free readers). Compaction can rebuild postings later without changing ids.
- Concurrency: immutable postings plus an atomic or lock-guarded deleted set; readers do not block each other.
- Memory: intern words, store ids as primitive arrays or delta-encoded varints; the index is usually smaller than the text.
- Phrase search needs positions per (word, line); prefix search needs a sorted word dictionary (or a trie) and a union of postings.

## Trade-off questions to ask when the candidate is done
- Tombstones versus physically removing ids from postings: which one is simpler under concurrency, and when does it stop paying off?
- If 90% of the lines are deleted, what changes about query cost, and when would you compact?
- How would `addLine` change your choice of data structures?
- What would you cache, and what invalidates the cache on `delete_line`?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.

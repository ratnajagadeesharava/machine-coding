# 109 · Document Line Search (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

A document is a list of text lines. Each line gets a stable integer id when the document is loaded, and users search the document for the lines that mention a set of words. You are building the search core: `search("cat dog")` returns the ids of every line that contains both words, a line can be deleted for good, and no id ever moves or gets reused. The same document is searched many times, so repeated searches have to stay fast on a large document.

## Problem

An editor loads a document once and then answers "which lines mention all of these words?" over and over (find-in-document, a log viewer, a notes app). Lines are identified by their position at load time: the first line is id 1, the second id 2, and so on. Users can delete lines; a deleted line must disappear from every later search, but the remaining lines keep their ids so that anything that stored an id (a bookmark, a comment anchor) still points at the same text.

## Requirements

1. `create(lines)` loads a document. The line at position `i` (counting from 1) gets id `i`. Ids are assigned once, never renumbered after a deletion and never reused. Blank lines are valid lines and take an id. Changes to the input list made after `create` do not affect the document.
2. A **word** is a maximal run of letters and digits in a line or in a query; every other character (space, punctuation, symbol, underscore, tab, newline) separates words. `search(query)` splits the query into words the same way.
3. `search(query)` returns the ids of the live lines that contain **every** distinct word of the query. Words may appear in any order and in any position in the line, and a line may repeat a word.
4. Matching is case-insensitive, and a query word matches only a whole word: `cat` matches `Cat` and `cat,` but not `category`, `cats` or `concat`. There is no stemming and no partial match.
5. Duplicate words in the query change nothing: `search("a a b")` equals `search("a b")`.
6. A query with no word in it (empty, blank, punctuation only) returns an empty list. It does not match every line.
7. The result is in increasing id order, contains each id once, and is a fresh list the caller may modify without affecting the document.
8. `delete_line(id)` removes a live line permanently and returns `true`. An id that was never assigned (zero, negative, past the last line) or that was already deleted returns `false` and changes nothing. All other lines keep their ids.
9. A deleted line never appears in any later search, whatever the query.
10. The service is used by many threads at once. Searches and deletions may overlap; a search sees each deletion either entirely or not at all, and when two threads delete the same id exactly one of them gets `true`.
11. Repeated searches on a moderately large document (20,000 lines) must be fast: the tests run thousands of searches against it with a generous time bound that a full scan with re-splitting of every line on every search will not meet.

## Constraints

- Thread-safe: concurrent `search` and `delete_line` calls, from any number of threads, never throw and never return a deleted id after the deletion has returned.
- No clock, no randomness, no I/O: the same document and the same calls always give the same answers.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Rust: there are no error values; `search` takes `&str`, ids are `u32`, `delete_line` returns `bool`.
- `search` with a query that has no words returns an empty list (not an error and not every line).
- `delete_line` returns `false` (it does not throw) for an unknown or already deleted id.
- Words are compared case-insensitively and as whole words; a line and a query are split into words by the same rule.
- The result is sorted by id ascending, without duplicates, and never contains a deleted id.
- A 20,000-line document answers 2,000 searches within a generous time bound.

## Examples

```
create(&["The cat sat", "A Category of cats", "cat, DOG!", "", "the dog and the cat"])   -> ids 1..=5
search("cat")          -> [1, 3, 5]       (line 2 has "category" and "cats", not the word "cat")
search("DOG Cat")      -> [3, 5]          (every word, any order, any case)
search("the the cat")  -> [1, 5]          (duplicates in the query change nothing)
search("")             -> []              search("   ") -> []          search("?!") -> []
delete_line(3)         -> true            search("cat") -> [1, 5]      (ids 1, 2, 4, 5 are unchanged)
delete_line(3)         -> false           delete_line(99) -> false     delete_line(0) -> false
```

```
create(&["alpha beta", "beta, gamma", "Alpha-Beta-Gamma", "alphabet soup"])
search("alpha")        -> [1, 3]          ("alphabet" is a different word)
search("beta gamma")   -> [2, 3]
search("gamma beta alpha") -> [3]
delete_line(1)         -> true
search("alpha")        -> [3]             search("beta") -> [2, 3]
```

```
create(&["v2 release notes", "release 2 notes", "snake_case name", "Ünïcode ÉCOLE"])
search("2")            -> [2]             ("v2" is one word; "2" is a different one)
search("snake case")   -> [3]             (an underscore is not a letter or digit, so it separates words)
search("école")        -> [4]             (case-insensitive for non-ASCII letters too)
```

### Edge cases

- `cat` does not match `category`, `cats`, `concat` or `bobcat`; it does match `cat.`, `(cat)`, `CAT`, `the-cat` and `cat_food` (underscore separates).
- `v2` is one word. The query `2` does not match it, and the query `v` does not either.
- A query word that appears in no line makes the whole result empty, even if the other words match many lines.
- A line that repeats a word (`"cat cat cat"`) appears once in the result.
- Deleting line 1 does not turn line 2 into line 1. Deleting the last line does not free its id.
- A blank line (`""`, `"   "`) has an id, matches no query, and can be deleted (`true` the first time).
- Deleting a line whose words are the only match for a query makes that query return an empty list.
- Inputs cannot be null in Rust.

## Glossary

- **Line**: one element of the list given to `create`; identified by its id.
- **Id**: the 1-based position a line had when the document was loaded; stable for the life of the document.
- **Word**: a maximal run of letters and digits; compared ignoring case.
- **Live line**: a line that has not been deleted.
- **Query**: the text passed to `search`; it is split into words by the same rule as the lines.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

pub trait DocumentSearch: Send + Sync {
    /// Ids of the live lines that contain every word of `query`, in increasing id order.
    /// Words are maximal runs of alphanumeric characters, compared case-insensitively and as whole words.
    /// A query without any word (empty, blank, punctuation only) returns an empty vector.
    fn search(&self, query: &str) -> Vec<u32>;

    /// Removes the line for good; every other line keeps its id and a deleted id is never reused.
    /// Returns true if the line was live, false if the id is unknown or was already deleted.
    fn delete_line(&self, id: u32) -> bool;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create<S: AsRef<str>>(lines: &[S]) -> Option<Arc<dyn DocumentSearch>> { ... }
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. There is no clock in this problem, so nothing sleeps. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Single and multi word queries, any order, any case, whole words only, punctuation, digits and underscores as separators.
- Duplicate query words, blank and punctuation-only queries, queries that match nothing, results in increasing order and independent of the document.
- Deletion: the id is gone from results, other ids are unchanged, unknown and repeated deletions return false, blank lines and the empty document.
- A 20,000-line document: correct counts for single and multi word queries and 2,000 searches within the time bound.
- Concurrency: 8 threads deleting different lines, 8 threads racing to delete the same line, searches running while lines are deleted.
- More edge cases (100,000-line documents, interleaved deletes and searches, non-ASCII text) are in `interviewer/`.

## Running

```bash
cargo test -p p109_document_line_search
STRICT=1 cargo test -p p109_document_line_search   # fail instead of skip while unimplemented
cargo test -p p109_document_line_search --features hidden   # interviewer scenarios
cargo play run 109             # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`run_concurrently`, `sut!`): `rust/support`.

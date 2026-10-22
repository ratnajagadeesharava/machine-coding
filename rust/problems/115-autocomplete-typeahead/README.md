# 115 · Autocomplete / Typeahead Service (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 75 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

When you type `ja` into a search box, the box suggests `java`, `javascript`, `jazz` before you finish typing. You are building the engine behind that box: it learns which search terms people use (every search is *recorded*), and for any prefix it returns the best few completions, fast. What "best" means can change: the most frequently searched, the most recently searched, or, for a signed-in user, the terms *that user* searches most often. Searches keep arriving from many servers while suggestions are being served, so a new count must be visible immediately and nothing may be lost under concurrency.

## Problem

Build a typeahead service. `record` tells it that a term was searched (once, or a number of times). `suggest(prefix, k)` returns up to `k` recorded terms that start with the prefix, best first under the active ranking rule. Three rules exist: **frequency** (highest count first), **recency** (most recently recorded first) and a **personalised** query, `suggestFor(user, prefix, k)`, which puts the terms that user has recorded most often ahead of everything else. The ranking rule is chosen when the service is created and can be switched later without losing any data. The service holds at least a hundred thousand terms and answers ten thousand suggestions in a few seconds.

## Requirements

1. Terms are stored trimmed and lower-cased; internal whitespace is kept (`"New York"` is one term, `new york`). A null or blank term is an error.
2. `record(term)` adds 1 to the term's count; `record(term, count)` adds `count`, which must be positive. Both mark the term as *recorded now* (see recency). A rejected call changes nothing, in particular it must not create the term.
3. `count(term)` returns the total recorded count of a term, 0 if it was never recorded.
4. `suggest(prefix, k)` returns at most `k` terms that start with the prefix, including a term equal to the prefix. The prefix is trimmed and lower-cased as well. An empty or blank prefix matches every term (the top-k overall). If nothing matches, including when the prefix is longer than every term, the result is an empty list. `k <= 0` and a null prefix are errors.
5. Frequency ranking: count descending. Recency ranking: the term recorded last first, where "when" is the injected clock's time at the moment of the `record` call; each new record of a term moves it to the front regardless of its count. Terms with equal rank (the same count, or recorded at the same clock instant) are ordered alphabetically, ascending, so every result is deterministic.
6. The ranking rule is passed at creation and can be changed with `setRanking`; counts and recency survive a switch. A null ranking is an error.
7. `recordFor(userId, term)` records the term once globally (count and recency, exactly like `record`) and also adds 1 to that user's own count for the term. A null or blank user id is an error.
8. `suggestFor(userId, prefix, k)` returns the same candidates as `suggest`, ordered by the user's own count of each term (descending; a term the user never recorded counts 0), then by the active ranking rule, then alphabetically. A user without history gets exactly what `suggest` returns. Users never see each other's counts.
9. A count update must be visible to the very next call from any thread. Concurrent writers must not lose updates (many threads recording the same term end with the exact total), and readers may run while writers are active.
10. Scale expectation: 100,000 distinct terms inserted and 10,000 `suggest` calls complete well within a generous time bound.

## Constraints

- The service must be thread-safe; the thread-safety tests use 16 writer threads on a handful of hot terms.
- `suggest` returns a fresh list that the caller may modify; the result never contains duplicates and never more than `k` entries.
- The fake clock is the only source of time; the tests never sleep.

## Examples

```
record("apple") x3, record("application") x5, record("apply")
suggest("app", 10)   -> [application, apple, apply]           (counts 5, 3, 1)
suggest("app", 2)    -> [application, apple]
suggest("", 1)       -> [application]                         (top-k overall)
suggest("zzz", 5)    -> []

record("pear"), record("peach"), record("pea"), record("pecan")  (all count 1)
suggest("pe", 10)    -> [pea, peach, pear, pecan]            (equal counts: alphabetical)
suggest("pea", 10)   -> [pea, peach, pear]                   (a term equal to the prefix is included)

setRanking(RECENCY), clock = 0
record("alpha", 100); clock = 10: record("alps"); clock = 20: record("alpine", 5)
suggest("al", 10)    -> [alpine, alps, alpha]                (count is irrelevant)
clock = 30: record("alpha")
suggest("al", 2)     -> [alpha, alpine]

record("java", 50), record("javascript", 40), record("jazz", 30)
recordFor("ann", "jazz") x2, recordFor("ann", "javascript")        (java 50, javascript 41, jazz 32 globally)
suggest("ja", 10)           -> [java, javascript, jazz]
suggestFor("ann", "ja", 10) -> [jazz, javascript, java]           (ann: jazz 2, javascript 1, java 0)
suggestFor("bob", "ja", 10) -> [java, javascript, jazz]           (no history: same as suggest)
```

### Edge cases

- `record("  Hello World ")` and `record("HELLO world")` are the same term `hello world`; `suggest(" HeL", 5)` finds it. Suggestions are returned in their stored form (trimmed, lower-case).
- A prefix is trimmed too: `suggest("cats ", 5)` behaves like `suggest("cats", 5)`.
- `record("x", 0)`, `record("x", -3)` and `record("  ")` are errors and leave the service exactly as it was (`count("x")` stays 0, `x` is never suggested).
- With the frequency rule, raising a low-ranked term above the leaders by one bulk `record(term, 1000)` is visible to the next `suggest`, for every prefix of that term, including the empty prefix.
- With the recency rule, terms recorded at the same clock instant tie (alphabetical); a bulk record and a `recordFor` both count as "recorded now".
- Switching the ranking back and forth never changes a count.
- In `suggestFor`, two terms with the same user count are ordered by the active ranking, then alphabetically; for an unknown user the answer equals `suggest`.
- `k` larger than the number of matches returns all matches.

## Glossary

- **Term**: a normalised search string, as stored.
- **Prefix**: the (normalised) leading characters typed so far; may be empty.
- **Count**: how many times a term was recorded in total (all users).
- **Recency**: the clock time of a term's most recent record.
- **Active ranking**: the rule set at creation or by `setRanking` that orders suggestions.
- **User count**: how many times one specific user recorded a term via `recordFor`.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Rust: `Result<_, TypeaheadError>` with the single variant `InvalidArgument`, returned for a blank term or user id, `count == 0` and `k == 0` (`record_count` takes a `u64`, `k` is a `usize`, so negative values cannot occur). `record(term)` is `record(term, 1)`; the overloaded Java `record(term, count)` is `record_count`. Time comes from the `Clock` handed to `create` (the tests pass a `FakeClock`), as does the initial `Ranking`.
- Normalisation: trim, then lower-case (locale-independent). Prefixes are normalised the same way. Alphabetical means ascending character order of the stored terms (the tests use ASCII and simple accented letters).
- Order of precedence in `suggest`: active ranking, then alphabetical. In `suggestFor`: user count, then active ranking, then alphabetical.
- Rejected calls leave no trace: no term is created, no count changes.
- `recordFor` also updates the global count and the global recency of the term.
- Reads and writes may be concurrent; there are no ordering guarantees between threads beyond "a completed call is visible to calls that start after it".

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Terms are stored trimmed and lower-cased; internal whitespace is kept. Every method that takes a term, prefix or
//! user id returns `InvalidArgument` for a blank term or user id (a blank prefix is valid and matches everything).
//! Time comes from the clock handed to `create` (epoch milliseconds).

#[derive(Debug, PartialEq, Eq)]
pub enum TypeaheadError {
    /// blank term / blank user id / zero count / `k == 0`
    InvalidArgument,
}

/// Rule used to order suggestions that start with the same prefix.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ranking {
    /// Higher recorded count first.
    Frequency,
    /// Most recently recorded first (time comes from the clock handed to `create`).
    Recency,
}

pub trait Typeahead: Send + Sync {
    /// Adds 1 to the count of `term` and marks it as recorded now. Same as `record_count(term, 1)`.
    fn record(&self, term: &str) -> Result<(), TypeaheadError>;

    /// Adds `count` (must be positive) to the count of `term` and marks it as recorded now. The new count is visible
    /// to the very next call, from any thread. A rejected call changes nothing.
    fn record_count(&self, term: &str, count: u64) -> Result<(), TypeaheadError>;

    /// Total count of `term` (0 if never recorded). Term is normalised like in `record`.
    fn count(&self, term: &str) -> Result<u64, TypeaheadError>;

    /// Switches the active ranking rule; counts and recency are kept.
    fn set_ranking(&self, ranking: Ranking);

    /// Up to `k` terms that start with `prefix` (a term equal to the prefix qualifies), best first under the active
    /// ranking: Frequency = count descending; Recency = most recently recorded first (terms recorded at the same
    /// clock instant are tied). Ties are always broken alphabetically (ascending, plain character order).
    /// The prefix is trimmed and lower-cased; an empty or blank prefix matches every term (top-k overall).
    /// Unmatched prefix -> empty list. `k == 0` -> `InvalidArgument`.
    fn suggest(&self, prefix: &str, k: usize) -> Result<Vec<String>, TypeaheadError>;

    /// Like `record` and additionally adds 1 to `user_id`'s own count of the term
    /// (the global count and recency are updated too).
    fn record_for(&self, user_id: &str, term: &str) -> Result<(), TypeaheadError>;

    /// Personalised suggestions: same candidates as `suggest`, but ordered by `user_id`'s own count of the term
    /// (descending; terms the user never recorded count 0), then by the active ranking, then alphabetically.
    /// A user with no history gets exactly the `suggest` result.
    fn suggest_for(
        &self,
        user_id: &str,
        prefix: &str,
        k: usize,
    ) -> Result<Vec<String>, TypeaheadError>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(clock: Arc<dyn Clock>, ranking: Ranking) -> Option<Arc<dyn Typeahead>> { ... }   // playground_support::Clock
```

## What the tests check

These tests are black-box: they only call the contract above, so any design behind it works. Time-dependent behaviour uses an injectable fake clock, never real sleeping. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Frequency order, alphabetical ties, prefix matching (including a term equal to the prefix), top-k, empty prefix, no match, prefix longer than every term.
- Normalisation (case, surrounding whitespace, multi-word terms), bulk counts, immediate visibility of updates.
- Invalid input: blank/null term, zero or negative count, zero or negative `k`, blank user, null ranking; rejected calls change nothing.
- Recency ranking with a fake clock, its alphabetical tie-break, and switching rules at runtime.
- Personalised suggestions, isolation between users, fallback to global for unknown users, personal records also counting globally.
- Concurrency: 16 threads recording must give exact counts; readers run while writers update.
- Scale: 100,000 terms and 10,000 suggest calls within a generous bound.
- More (stale top-k after a bulk update, a 1,000-way tie, randomised comparison with a brute-force model, long terms, 200,000 terms) are in `interviewer/`.

## Running

```bash
cargo test                          # from this directory (or `cargo test -p p115_autocomplete_typeahead` from rust/)
STRICT=1 cargo test                 # fail instead of skip while unimplemented
cargo test --features hidden        # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation print `SKIPPED` and return (they do not fail) until `create` is wired up, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`FakeClock`, `sut!`, `run_concurrently`): `rust/support`.

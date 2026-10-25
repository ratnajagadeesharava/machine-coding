# 138 · Multi-Threaded Page Visit Counter (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

A website wants to know how often each of its pages is visited and by how many different people. Web servers on many threads report "visitor V opened page U" and dashboards ask questions at the same time: how many visits did this page get, how many distinct visitors, who are the most frequent visitors, which pages are the busiest, and what does the whole site look like right now. The numbers are **exact** (no sampling, no estimates) and **lifetime totals** (there are no time windows). Two spellings of the same address count as the same page, and a consistent point-in-time export must be possible while visits keep arriving.

## Problem

Build the counting core of a page analytics service. `visit(url, visitorId)` records one visit. Reads return exact totals, exact distinct-visitor counts, per-visitor counts and rankings. A snapshot returns a consistent copy of everything: even while 16 threads keep visiting, a snapshot must look like the state after some prefix of one serial order of all the visits that have happened, never a blend of an earlier state for one page and a later state for another. One page may receive a million visits from only a hundred visitors; memory must not depend on the number of visits, only on the number of distinct (page, visitor) pairs.

## Requirements

1. `visit(url, visitorId)` records one visit by `visitorId` to the page `url`, after normalising the url (below). A blank url, a url with an empty host, or a blank visitor id is an error and records nothing.
2. `totalVisits(url)` is the number of visits to the page, `0` if the page has no visits.
3. `uniqueVisitors(url)` is the number of distinct visitor ids that visited the page (exact), `0` for an unknown page.
4. `visitsBy(url, visitorId)` is how many times that visitor visited that page, `0` if never.
5. `topPages(k)` returns the `k` most visited pages as `"url:total"` (normalised url), ordered by total descending, then url ascending (plain string comparison). Fewer than `k` entries if there are fewer pages. `k` below 1 is an error.
6. `topVisitors(url, k)` returns the `k` most frequent visitors of that page as `"visitorId:count"`, ordered by count descending, then visitor id ascending (plain string comparison). Empty for an unknown page. `k` below 1 is an error.
7. The grand total (all visits to all pages) and `pageCount()` (pages that currently have at least one visit) are available.
8. `reset(url)` forgets that page entirely (its total, its visitors) and returns whether the page existed; `resetAll()` forgets everything. A forgotten page starts from zero if it is visited again.
9. `snapshot()` returns an immutable copy of all pages as `"url:total:unique"` lines sorted by url ascending (plain string comparison of the url). `snapshotDetail()` returns `"url:visitorId:count"` lines sorted by url ascending, then visitor id ascending. Each is one consistent point-in-time view.

### URL normalisation

Applied, in this order, to the `url` argument of every operation:

1. Trim leading and trailing whitespace. If nothing remains, the url is blank (error).
2. Remove the fragment: everything from the first `#` on.
3. Remove the query string: everything from the first `?` on (after step 2).
4. If the text contains `://`, the part before the first `://` is the **scheme** and the rest is the remainder; otherwise there is no scheme and the whole text is the remainder.
5. The **authority** is the remainder up to (not including) the first `/`, or all of it if there is no `/`. The **path** is the rest (starting with `/`), possibly empty. If the authority is empty the url is invalid (error).
6. Lower-case the scheme and the whole authority (host, and port or user-info if present). The path keeps its case.
7. Remove **all** trailing `/` characters of the path. If nothing is left (the path was empty or only slashes) the path becomes the single character `/`.
8. The normalised url is `scheme + "://"` (omitted when there was no scheme) + authority + path.

Default ports are not removed and `http` / `https` are different pages; `www.` is not stripped; nothing inside the path is decoded or collapsed.

## Constraints

- Many threads call `visit` while others read, rank, snapshot and reset. The counter must be thread-safe and must not deadlock.
- No lost updates: after 16 threads x 20,000 visits over 50 urls and 1,000 visitors every total, unique count and per-visitor count is exact.
- A snapshot must be consistent across pages (see Problem). Totals in successive snapshots never decrease while nobody resets, and in every snapshot each page has `total >= unique`.
- Memory is proportional to the number of distinct (page, visitor) pairs, not to the number of visits: 1,000,000 visits from 100 visitors on one page must be cheap (hidden: far more).
- Counts are exact: no approximation of the distinct-visitor count.

## Examples

```
visit("https://Example.com/about/?utm=mail#team", "u1")
visit("https://example.com/about", "u2")
visit("https://example.com/about", "u1")
totalVisits("https://example.com/about")      -> 3        (one page, three spellings)
uniqueVisitors("https://example.com/about")   -> 2
visitsBy("https://example.com/about", "u1")   -> 2
topVisitors("https://example.com/about", 5)   -> ["u1:2", "u2:1"]
```

```
visit("https://s.com/a", "x"); visit("https://s.com/a", "y"); visit("https://s.com/a", "x")
visit("https://s.com/b", "x");  visit("https://s.com/c", "y");  visit("https://s.com/c", "z")
topPages(2)        -> ["https://s.com/a:3", "https://s.com/c:2"]
topPages(9)        -> ["https://s.com/a:3", "https://s.com/c:2", "https://s.com/b:1"]
snapshot()         -> ["https://s.com/a:3:2", "https://s.com/b:1:1", "https://s.com/c:2:2"]
snapshotDetail()   -> ["https://s.com/a:x:2", "https://s.com/a:y:1", "https://s.com/b:x:1",
                       "https://s.com/c:y:1", "https://s.com/c:z:1"]
reset("https://s.com/a/")  -> true        reset("https://s.com/a") -> false
totalVisits()      -> 3                   pageCount() -> 2
```


```
Normalisation:
"  HTTP://Shop.COM  "                  -> "http://shop.com/"
"https://shop.com//"                   -> "https://shop.com/"
"https://shop.com/Docs/Intro///?x=1"   -> "https://shop.com/Docs/Intro"
"Shop.com/Docs/"                       -> "shop.com/Docs"           (no scheme)
"https://shop.com:8443/a#frag"         -> "https://shop.com:8443/a"
"https://shop.com/a" vs "https://shop.com/A" -> two different pages
"?a=1", "https:///x", "   "            -> error
```

### Edge cases

- A failed call (blank url, blank visitor) changes nothing. Every operation that takes a url validates it, including the reads (`totalVisits(" ")` is an error, not 0).
- Visitor ids are compared exactly as given: `"Bob"`, `"bob"` and `" bob"` are three visitors. Only blank ids are rejected.
- `topPages(k)` with more pages than exist returns all of them; with no pages at all it returns an empty list (not an error). `topVisitors` of an unknown page is empty.
- `reset` of a url that normalises to an existing page resets that page (`reset(".../a/?x=1")` resets `.../a`).
- After `reset`, `uniqueVisitors` and every per-visitor count of that page are `0`, and it disappears from `topPages`, `snapshot` and `pageCount`.
- A snapshot is a copy: visits and resets after it was taken never change it.
- The empty counter has an empty snapshot, grand total `0` and page count `0`.

## Glossary

- **Normalised url**: the canonical spelling of a page address (rules above); the identity of a page.
- **Visit**: one `(url, visitorId)` event. Visits are never deduplicated in `totalVisits`.
- **Unique visitors**: the number of distinct visitor ids for one page.
- **Snapshot**: an immutable copy of all counters corresponding to one point in a serial order of the visits.
- **Serial prefix**: the state you would get by applying only the first N visits of some one-at-a-time ordering of all visits.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- `Result<_, CounterError>` with the single variant `InvalidArgument` for a blank url, a url whose host is empty after normalisation, a blank visitor id and `k == 0`.
- Rust: `u64` counts, `usize` for `k` and `page_count`, `Vec<String>` for the rankings and exports, and `grand_total()` instead of the Java no-argument `totalVisits()`. Method names are snake_case (`total_visits`, `unique_visitors`, `visits_by`, `top_pages`, `top_visitors`, `reset_all`, `snapshot_detail`).
- Output formats: `topPages` -> `"url:total"`; `topVisitors` -> `"visitorId:count"`; `snapshot` -> `"url:total:unique"`; `snapshotDetail` -> `"url:visitorId:count"` (the url always contains a `:` when it has a scheme or port, so tests split from the right).
- Ordering: counts descending then name ascending for rankings; url ascending (then visitor id ascending) for snapshots; "ascending" is plain string comparison (byte/UTF-16 order, no locale).
- Snapshot consistency: if one thread alternately visits page X then page Y, then in every snapshot `0 <= total(X) - total(Y) <= 1`.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below. It is **not** your domain model — it is the doorway. Create whatever types you like and wire them up behind it.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

#[derive(Debug, PartialEq, Eq)]
pub enum CounterError {
    /// blank url, url with an empty host after normalisation, blank visitor id, or `k == 0`
    InvalidArgument,
}

/// Every `url` argument is normalised first (see the README). Visitor ids are compared exactly as given
/// (case-sensitive, not trimmed).
pub trait VisitCounter: Send + Sync {
    /// Records one visit of `visitor_id` to the page.
    fn visit(&self, url: &str, visitor_id: &str) -> Result<(), CounterError>;

    /// Number of visits to the page, 0 if it was never visited (or was reset).
    fn total_visits(&self, url: &str) -> Result<u64, CounterError>;

    /// Number of distinct visitor ids that visited the page. Exact. 0 for an unknown page.
    fn unique_visitors(&self, url: &str) -> Result<u64, CounterError>;

    /// Number of visits to the page by this visitor, 0 if none.
    fn visits_by(&self, url: &str, visitor_id: &str) -> Result<u64, CounterError>;

    /// The `k` most visited pages as `"url:total"` (normalised url), ordered by total descending then url
    /// ascending (plain string order). Fewer entries if there are fewer pages. `k == 0` is an error.
    fn top_pages(&self, k: usize) -> Result<Vec<String>, CounterError>;

    /// The `k` most frequent visitors of the page as `"visitorId:count"`, ordered by count descending then
    /// visitor id ascending. Empty for an unknown page. `k == 0` is an error.
    fn top_visitors(&self, url: &str, k: usize) -> Result<Vec<String>, CounterError>;

    /// Total number of visits over all pages.
    fn grand_total(&self) -> u64;

    /// Number of pages that currently have at least one recorded visit.
    fn page_count(&self) -> usize;

    /// Forgets the page and all its counts. Returns whether the page existed.
    fn reset(&self, url: &str) -> Result<bool, CounterError>;

    /// Forgets every page.
    fn reset_all(&self);

    /// A point-in-time view of all pages as `"url:total:unique"` lines sorted by url ascending. It is an owned copy:
    /// later visits do not change it, and it equals the state after some prefix of the serial history of all
    /// visits (never a page counted "ahead of" another page that was updated earlier).
    fn snapshot(&self) -> Vec<String>;

    /// Like `snapshot`, but per visitor: `"url:visitorId:count"` lines sorted by url then visitor id (plain string
    /// order). Taken at a single point in time as well.
    fn snapshot_detail(&self) -> Vec<String>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create() -> Option<Arc<dyn VisitCounter>>
```

## What the tests check

These tests are black-box: they only call the contract above, so any design behind it works. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Totals, uniques and per-visitor counts; unknown pages; visitor ids case-sensitive and untrimmed.
- Normalisation: case of scheme/host, query, fragment, whitespace, root path, repeated trailing slashes, no scheme, path case kept, different scheme/port/path are different pages.
- Errors: blank / null url, host-less url, blank visitor, `k` below 1, on every operation.
- Rankings with ties, `k` larger than the data, empty counter; grand total, page count, reset, reset of an unknown page, reuse after reset, reset all.
- Snapshot format, sort order, immutability.
- Concurrency: 16 threads x 20,000 visits (`run_concurrently`, 60 s watchdog) with exact totals, uniques and per-visitor counts; snapshots taken while 12 writers visit (pair invariant, `total >= unique`, monotonic); per-visitor sums that add up; a mix of writers, readers, rankers, snapshotters and resetters (no deadlock); 1,000,000 visits from 100 visitors on one page.
- More (memory growth, pathological ties, resets racing visits, ordered chains) are in `interviewer/`.

## Running

```bash
cargo test -p p138_page_visit_counter                    # from rust/
STRICT=1 cargo test -p p138_page_visit_counter           # fail instead of skip while unimplemented
cargo test -p p138_page_visit_counter --features hidden # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation print `SKIPPED` and return (not fail) until `create` returns `Some`, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`sut!`, `run_concurrently`): `rust/support`.

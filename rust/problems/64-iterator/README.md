# 64 · Iterator (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — `Page`, `PageFetcher` and `FeedError` are provided; `PagedFeed`, `FeedIter` and `FilterLazy` are stubs. |

## Problem

A social app shows a feed from a paginated REST API. Each page costs a network round-trip, so the feed must
fetch pages **only when needed**: a user who scrolls past 3 items should not trigger the download of 50 pages.

**What you are building:** a `PagedFeed` that behaves like one long sequence of strings (you can loop over it),
although the data really lives on a remote server in pages. Behind the scenes it asks the server for a page only
at the moment the reader actually needs an item from it. You also build `filter_lazy`, a small adapter that wraps
any iterator and hands out only the elements you want, again without reading ahead more than necessary.

### Provided types

- `Page { items: Vec<String>, has_next: bool }` — one page of results; `has_next` says whether the server has a page after this one.
- `PageFetcher::fetch(&self, page: usize) -> Result<Page, FeedError>` — the remote API. Page numbers start at 0. A page may be **empty even when `has_next` is true**.
- `FeedError(pub String)` — the error type returned when a fetch fails.

### Glossary

- **Lazy**: no work is done until a result is actually asked for.
- **Pull**: calling `next()` on an iterator to get something from it.

## Requirements

1. `PagedFeed::new(Rc<dyn PageFetcher>)`; `iter()` returns a new independent `FeedIter` starting at page 0 (each iterator fetches for itself). Nothing is fetched by either call.
2. `FeedIter` implements `Iterator<Item = Result<String, FeedError>>`. It is **lazy**: the first `next()` fetches page 0; page `n+1` is fetched only when page `n`'s items are used up *and* `next()` is called again. Merely returning the last item of page `n` does not fetch page `n+1`.
3. Empty pages are skipped (fetch the following page, and again, until a non-empty page or the end). After the last item `next()` returns `None` (and keeps doing so without fetching again).
4. If the fetcher fails, `next()` returns `Some(Err(error))` carrying the fetcher's own `FeedError`; the iterator stays usable and the next call **retries the same page** (it is fetched again, not skipped).
5. `filter_lazy(inner, keep)` returns a `FilterLazy` iterator yielding only the elements for which `keep` returns `true`. It never pulls from `inner` further than the element it is about to return: nothing is pulled until `next()` is called, and it stops pulling as soon as it has found one accepted element. When `inner` is exhausted it returns `None`.

## Examples

Pages: `0 = [a, b]`, `1 = [c, d]`, `2 = [e]`. "fetched" lists the page numbers requested so far.

| Call | Result | fetched |
|---|---|---|
| `it = feed.iter()` | — | `[]` |
| `it.next()` | `Some(Ok("a"))` | `[0]` |
| `it.next()` | `Some(Ok("b"))` | `[0]` (page 0 is used up, but nobody asked for more yet) |
| `it.next()` | `Some(Ok("c"))` | `[0, 1]` |

`feed.iter().take(3)` over pages `[a,b] [c,d] [e,f]` yields `a, b, c` and leaves fetched = `[0, 1]`.

Empty pages `[] [a] [] [] [b]`: iterating gives `a`, `b`, and fetched = `[0, 1, 2, 3, 4]`.

Failure and retry (pages `[a,b] [c]`, fetching page 1 fails with `FeedError("remote error on page 1")`):

```
it.next() -> Some(Ok("a"))
it.next() -> Some(Ok("b"))
it.next() -> Some(Err(FeedError("remote error on page 1")))
// fetcher recovers
it.next() -> Some(Ok("c"))        // fetched = [0, 1, 1]
```

Lazy filter over `1..=10` keeping even numbers (`pulled` = how many elements were taken from the source):

```
evens = filter_lazy(source, |n| n % 2 == 0)   // pulled = 0
evens.next() -> Some(2)                        // pulled = 2 (read 1, rejected; read 2, kept)
evens.next() -> Some(4)                        // pulled = 4
```

## Edge cases

- A feed with a single page and a single item: `Some(Ok(item))`, then `None`, `None`, ...; only page 0 was fetched.
- Filtering a feed composes: pages `[aa,b,cc] [d,ee]` mapped to the strings and filtered by "length is 2" gives `aa, cc, ee`, then `None`.
- If every remaining element is rejected, `filter_lazy` returns `None` after exhausting the source.

## Running

```bash
cargo test -p p64_iterator     # from rust/ — tests FAIL until you implement the stubs
cargo play run 64
```

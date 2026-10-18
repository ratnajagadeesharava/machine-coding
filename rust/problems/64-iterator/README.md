# 64 · Iterator (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — `Page`, `PageFetcher` and `FeedError` are provided; `PagedFeed`, `FeedIter` and `FilterLazy` are stubs. |

> **Pattern:** *Iterator* — give clients a uniform way to walk a collection **without exposing how it is stored**.
> In Rust the pattern is the `Iterator` trait itself: implement `next()` and every adapter (`take`, `map`,
> `collect`...) works for free. Here the "collection" is a remote, paginated API fetched lazily.

## Problem

A social app shows a feed from a paginated REST API. Each page costs a network round-trip, so the feed must
fetch pages **only when needed**: a user who scrolls past 3 items should not trigger the download of 50 pages.

## Requirements

1. `PagedFeed::new(Rc<dyn PageFetcher>)`; `iter()` returns a new independent `FeedIter` starting at page 0. Nothing is fetched by either call.
2. `FeedIter` implements `Iterator<Item = Result<String, FeedError>>`. It is **lazy**: the first `next()` fetches page 0; page `n+1` is fetched only when page `n`'s items are used up *and* `next()` is called again.
3. Empty pages are skipped (fetch the following page). After the last item `next()` returns `None` (and keeps doing so without fetching again).
4. If the fetcher fails, `next()` returns `Some(Err(error))`; the iterator stays usable and the next call **retries the same page**.
5. `filter_lazy(inner, keep)` returns a `FilterLazy` iterator that never pulls from `inner` further than the element it is about to return. (It is `Iterator::filter` written by hand; use the stub's `PhantomData` field as a starting point and replace it with your own fields.)

## Constraints (what the tests catch)

- The API is a fake that counts fetches and can fail on a chosen page; `inspect` on a source iterator counts how many elements the filter pulled.
- `take(3)` on a feed with pages of 2 must fetch exactly two pages.

## Running

```bash
cargo test -p p64_iterator     # from rust/ — tests FAIL until you implement the stubs
cargo play run 64
```

# 64 · Iterator (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — `Page` and `PageFetcher` are provided; `PagedFeed` and `Iterators` are stubs. |

## Problem

A social app shows a feed from a paginated REST API. Each page costs a network round-trip, so the feed must
fetch pages **only when needed**: a user who scrolls past 3 items should not trigger the download of 50 pages.

## Requirements

1. `PagedFeed(PageFetcher)` is an `Iterable<String>`. Constructing it or calling `iterator()` fetches nothing.
2. Its iterator yields all items of all pages in order, and is **lazy**: the first `hasNext()`/`next()` fetches page 0; page `n+1` is fetched only when page `n`'s items are used up *and* the consumer asks again.
3. Empty pages are skipped (fetch the following page). `hasNext()` is idempotent; `next()` at the end throws `NoSuchElementException`.
4. Every `iterator()` call returns a new independent iterator starting at page 0.
5. If the fetcher throws, the exception propagates from `hasNext()`/`next()`; the iterator stays usable and the next attempt **retries the same page**.
6. `Iterators.filter(source, predicate)` is a lazy filtering iterator over any iterator that never pulls more than it needs; `hasNext()` is idempotent and `next()` at the end throws `NoSuchElementException`. `null` source or predicate → `IllegalArgumentException`.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

# 64 · Iterator (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — `Page` and `PageFetcher` are provided; `PagedFeed` and `Iterators` are stubs. |

## Problem

A social app shows a feed from a paginated REST API. Each page costs a network round-trip, so the feed must
fetch pages **only when needed**: a user who scrolls past 3 items should not trigger the download of 50 pages.

**What you are building:** a `PagedFeed` that looks like one long list of strings (you can loop over it with a
`for` loop), although the data really lives on a remote server in pages. Behind the scenes it asks the server for
a page only at the moment the reader actually needs an item from it. You also build a small utility,
`Iterators.filter`, that wraps any iterator and hands out only the elements you want, again without reading
ahead more than necessary.

### Provided types

- `Page(List<String> items, boolean hasNext)` — one page of results; `hasNext` says whether the server has a page after this one.
- `PageFetcher.fetch(int pageNumber)` — the remote API. Page numbers start at 0. A page may be **empty even when `hasNext` is true**. It may throw a `RuntimeException`.

### Glossary

- **Lazy**: no work is done until a result is actually asked for.
- **Idempotent**: calling it several times in a row has the same effect as calling it once (here: `hasNext()` never skips or consumes an item and never causes an extra fetch once the answer is known).
- **Pull**: calling `next()` (or `hasNext()`) on an iterator to get something from it.

## Requirements

1. `new PagedFeed(PageFetcher)` is an `Iterable<String>`. Constructing it or calling `iterator()` fetches nothing.
2. Its iterator yields all items of all pages in order, and is **lazy**: the first `hasNext()`/`next()` fetches page 0; page `n+1` is fetched only when page `n`'s items are used up *and* the consumer asks again. Merely consuming the last item of page `n` does not fetch page `n+1`.
3. Empty pages are skipped (fetch the following page, and again, until a non-empty page or the end). `hasNext()` is idempotent; `next()` at the end throws `NoSuchElementException`. At the end the fetcher is not called again.
4. Every `iterator()` call returns a new independent iterator starting at page 0 (each one fetches for itself).
5. If the fetcher throws, the exception propagates unchanged (same exception object) from `hasNext()`/`next()`, whichever call needed the page; the iterator stays usable and the next attempt **retries the same page** (it is fetched again, not skipped).
6. `Iterators.filter(source, predicate)` is a lazy filtering iterator over any iterator that never pulls more than it needs: nothing is pulled from `source` until `hasNext()`/`next()` is called, and it stops pulling as soon as it has found one accepted element. `hasNext()` is idempotent (calling it repeatedly does not pull further elements) and `next()` at the end throws `NoSuchElementException`. `null` source or predicate → `IllegalArgumentException` (thrown by `filter` itself).

## Examples

Pages: `0 = [a, b]`, `1 = [c, d]`, `2 = [e]`. "fetched" lists the page numbers requested so far.

| Call | Result | fetched |
|---|---|---|
| `it = feed.iterator()` | — | `[]` |
| `it.next()` | `"a"` | `[0]` |
| `it.next()` | `"b"` | `[0]` (page 0 is used up, but nobody asked for more yet) |
| `it.next()` | `"c"` | `[0, 1]` |

Breaking out of a `for` loop after 3 items of pages `[a,b] [c,d] [e,f]` leaves fetched = `[0, 1]`.

Empty pages `[] [a] [] [] [b]`: iterating gives `a`, `b`, and fetched = `[0, 1, 2, 3, 4]`.

Failure and retry (pages `[a,b] [c]`, fetcher throws `IllegalStateException("remote error on page 1")` for page 1):

```
it.next() -> "a"
it.next() -> "b"
it.hasNext()  -> throws IllegalStateException (message mentions page 1)
// fetcher recovers
it.hasNext()  -> true
it.next()     -> "c"          // fetched = [0, 1, 1]
```

Lazy filter over `1..10` keeping even numbers (`pulled` = how many elements were taken from the source):

```
evens = Iterators.filter(source, n -> n % 2 == 0)   // pulled = 0
evens.hasNext() -> true, again -> true               // pulled = 2 (read 1, rejected; read 2, kept)
evens.next()    -> 2                                 // pulled = 2
evens.next()    -> 4                                 // pulled = 4
```

## Edge cases

- A feed with a single page and a single item: `hasNext()` true, `next()`, then `hasNext()` false (repeatedly) and `next()` throws `NoSuchElementException`; only page 0 was fetched.
- Filtering a `PagedFeed` iterator composes: pages `[aa,b,cc] [d,ee]` with "length is 2" gives `aa, cc, ee`.
- If every remaining element is rejected, the filter's `hasNext()` returns `false` after exhausting the source.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

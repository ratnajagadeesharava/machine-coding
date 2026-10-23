# 118 · Search Result Cache (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 75 min | Level 2 — the backend interface, the cache entry point and the tests are provided; the cache design behind them is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

A search service is slow and expensive, and users keep asking for the same things ("red shoes", "Red  Shoes", "shoes red"). You are building the **result cache** that sits in front of the search backend. It decides when two spellings of a query are the same, remembers results for a limited time and a limited number of queries, throws results away as soon as the index they came from has changed, and makes sure that a burst of identical queries costs the backend one call, not a thousand. (A generic key/value cache is a different exercise; this one is about the query, the index version and the in-flight call.)

## Problem

The backend (provided by the tests as a counting fake) answers `search(query)` with a list of document ids and reports an `indexVersion` that goes up whenever the index changes. Build a cache that is created with a backend, a capacity, a time-to-live and a clock, and answers `get(rawQuery)`. A cached answer is only good while it is young enough, while the index is unchanged and while it has not been pushed out by more recent queries. When many threads ask the same thing while the backend is still working on it, the backend must be asked once and everybody gets that answer; when that one call fails, everybody learns about it and nothing bad is remembered.

## Requirements

1. **Normalisation.** Two raw queries are the same query if, after trimming, lower-casing and collapsing every run of whitespace (spaces, tabs, newlines) into one space, they are equal text. Word order matters: `red shoes` and `shoes red` are different. The normalised text is the cache key and is exactly what the backend receives.
2. `get(rawQuery)` returns the backend's document ids for the query, in the backend's order. A null or blank query is an error and changes no statistic.
3. **Hit.** A stored entry is served only if (a) it is younger than the TTL: it expires exactly `ttl` ms after it was stored, and hits do not extend it; and (b) the backend's `indexVersion` is still equal to the version read **immediately before** the backend call that produced the entry. Otherwise the entry is dropped and the backend is called again; the new entry gets a full new TTL.
4. **Capacity and LRU.** The cache never holds more entries than its capacity. When a new entry would exceed it, the least recently used entry is dropped. Storing an entry and serving a hit both count as use.
5. **Invalidation.** `invalidate(rawQuery)` drops the entry of that (normalised) query, `invalidateAll()` drops everything. Both are fine for queries that are not cached; `invalidate` rejects a null/blank query. A backend call that was already running when an invalidation covering its query happened must not be stored, and a `get` issued after the invalidation must not receive the result of such a call (it starts a fresh call).
6. **Request coalescing.** If a backend call for a normalised query is in flight, other `get`s for the same normalised query (whatever their spelling) wait for it and receive the same result; the backend is invoked once. Different queries never wait for each other.
7. **Failures.** If the backend call throws, every caller that waited for that call gets that failure; nothing is stored; the next `get` calls the backend again. Empty results are ordinary results and are cached.
8. **Returned lists** are unmodifiable (Java) / owned by the caller (Rust) and independent of whatever list the backend returned (the backend may keep changing it).
9. **Statistics.** `hits()` counts gets served from a stored entry. `misses()` counts gets that were not (including those that joined an in-flight call and those whose call failed); a get is counted as soon as it has decided it cannot be served from the cache, before it waits. Every valid `get` is exactly one hit or one miss. `backendCalls()` counts invocations of the backend's search (failed ones included). `size()` is the number of entries a `get` would serve right now (so expired entries and entries of an older index version are not counted).
10. **Configuration.** Capacity and TTL must be positive; the backend and clock must be present.

## Constraints

- Many threads call `get`, `invalidate*` and the statistics concurrently. The cache must be thread-safe and must never lose or double-count a statistic.
- A slow backend call for one query must not block hits or calls for other queries.
- All time-dependent behaviour uses the injected clock. Tests never sleep to move time.
- `indexVersion` only ever goes up.

## Examples

```
cache(capacity=2, ttl=1000ms)
get("Red  SHOES")   -> backend.search("red shoes") -> [d1, d2]      hits=0 misses=1 calls=1
get("red shoes")    -> [d1, d2]  (from cache)                       hits=1 misses=1 calls=1
t += 1000ms
get("red shoes")    -> backend called again (expired at exactly ttl)  calls=2

get("a"); get("b"); get("a"); get("c")   // capacity 2: "b" was least recently used
get("a") -> hit     get("b") -> backend call
```

```
get("q") -> [x]           backend.indexVersion goes 7 -> 8
size()   -> 0             get("q") -> backend called again, entry now stored at version 8
```

```
8 threads call get("red shoes") / get("Red Shoes") while the backend call is blocked
-> all 8 are counted as misses, backend.search was invoked once, all 8 receive the same list
backend throws "boom" -> all 8 get "boom", size()=0, the next get calls the backend again
```

### Worked traces

```
get("red shoes") twice            -> backend called once; hits=1 misses=1
get("  Red   SHOES ")             -> backend receives "red shoes"
get("shoes red")                  -> a different query (word order matters)
get("  ")                         -> Err(InvalidQuery), nothing counted
index version changes while search("q") is running (version read before the call: 3, after: 4)
  -> the result is returned to the caller; the entry stored for it carries version 3, so the next get() is a miss
invalidate("Q") while a call for "q" is in flight, then get("q") -> a second backend call; the first call's result is not stored
```

### Edge cases

- Tabs, newlines and repeated spaces collapse to a single space; leading/trailing whitespace is dropped; case is folded (non-ASCII letters too).
- `ttl` boundary: age `ttl - 1` is a hit, age `ttl` is a miss.
- A hit refreshes recency but never the TTL.
- Reloading an expired or stale entry replaces it; it does not count twice towards capacity.
- Expired / stale entries do not count in `size()`, even if they have not been physically removed.
- An empty result list from the backend is cached like any other.
- Invalid arguments change no statistic and no entry.

## Glossary

- **Normalised query**: the canonical text of a query (trimmed, lower-cased, single spaces, same word order).
- **Index version**: a number published by the backend that goes up whenever the searchable data changed.
- **TTL**: how long, in milliseconds, a stored result may be served.
- **In flight**: a backend call that has started and not yet returned.
- **Coalescing**: serving several concurrent identical requests from one backend call.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Rust: `get` returns `Result<Vec<String>, CacheError>`; `CacheError::InvalidQuery` for a blank query (in `get` and `invalidate`); `CacheError::Backend(SearchError)` when the backend call failed (every waiter gets an equal error). `create` returns `None` for a zero capacity or zero ttl (after a valid configuration has shown the crate is wired, `None` means "rejected").
- The backend is the `SearchBackend` trait (`search(&self, &str) -> Result<Vec<String>, SearchError>`, `index_version(&self) -> u64`), implemented by the tests. The clock is `playground_support::Clock`.
- Entries are stamped with the clock reading when the backend call returned and with the index version read just before the call.
- `hits + misses` equals the number of valid `get` calls; `backendCalls <= misses`.
- Waiters are counted as misses on arrival: the tests wait for `misses()` to reach the number of callers to know that every caller is blocked on the one call.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract` - see the source). It is **not** your domain model — it is the doorway. Add your own types next to it and wire them up.

```rust
pub struct SearchError(pub String);

pub enum CacheError {
    InvalidQuery,
    Backend(SearchError),
}

pub trait SearchBackend: Send + Sync {          // PROVIDED, implemented by the tests
    fn search(&self, normalised_query: &str) -> Result<Vec<String>, SearchError>;
    fn index_version(&self) -> u64;             // only ever goes up
}

pub trait SearchCache: Send + Sync {
    fn get(&self, raw_query: &str) -> Result<Vec<String>, CacheError>;
    fn invalidate(&self, raw_query: &str) -> Result<(), CacheError>;
    fn invalidate_all(&self);
    fn hits(&self) -> u64;
    fn misses(&self) -> u64;
    fn backend_calls(&self) -> u64;
    fn size(&self) -> usize;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(backend: Arc<dyn SearchBackend>, capacity: usize, ttl_millis: u64,
              clock: Arc<dyn Clock>) -> Option<Arc<dyn SearchCache>> { ... }
```

## What the tests check

These tests are black-box: they only call the contract above, so any design behind it works. The backend is a counting fake with a bumpable index version and a latch that can hold a call open; time moves only through a fake clock. More scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Miss then hit, statistics, normalisation (case, tabs, newlines), word order, what the backend receives.
- TTL boundary, no sliding TTL, full TTL after a reload.
- LRU eviction order and the capacity bound.
- Index-version invalidation, `invalidate`, `invalidateAll`, `size()` ignoring dead entries.
- Invalid queries and configuration, unmodifiable/independent results, empty results.
- Backend failure propagated and not cached; coalescing of 8 identical concurrent queries into one call (and failure to 6 waiters); a slow call not blocking other queries; a mixed concurrent load with statistics consistency.
- `tests/behavior.rs` (25 tests) and `interviewer/hidden.rs` (9 scenarios).

## Running

```bash
cargo test -p p118_search_result_cache
STRICT=1 cargo test -p p118_search_result_cache            # fail instead of skip while unimplemented
cargo test -p p118_search_result_cache --features hidden   # interviewer scenarios
cargo play run 118                                         # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`FakeClock`, `sut!`, `eventually`): `rust/support`.

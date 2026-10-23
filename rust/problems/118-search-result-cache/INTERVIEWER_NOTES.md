# Interviewer notes — 118 Search Result Cache

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, concurrency bug, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Decisions fixed by the contract (candidates often ask)
- Key = trim + lower-case + collapse whitespace, word order preserved (no token sorting). Sorting tokens would merge "new york" and "york new"; some products want that, this one does not.
- Entry age is measured from when the backend call **returned**; the entry version is read **before** the call. Reading it after the call would let an index change during the call be stamped as current and serve stale data for a whole TTL.
- Waiters are misses, counted on arrival. `size()` counts only live entries.
- `indexVersion()` is monotonic; the check is equality (differs = stale), so a rollback of the version number is out of scope.

## Hidden edge cases (see `interviewer/`)
- Index version bumped by the backend itself during the call: the stored entry must not be served afterwards.
- `invalidate`/`invalidateAll` while a call is in flight: the late result is returned to its own waiters but never stored over a newer entry, and a `get` made after the invalidation starts its own call.
- 32 threads x 500 overlapping keys: exactly 500 backend calls.
- Every waiter gets the very same exception object (Java) / an equal error (Rust); one failing query does not poison others.
- Eviction order after a mix of hits and reloads; capacity 1 under 16 threads with the version and clock changing.
- Non-ASCII lower-casing (`ÉCOLE` -> `école`), no dependence on the default locale (use `Locale.ROOT`).
- Statistics: `hits + misses` must equal the number of valid gets even when threads race and expire entries.

## Likely interviewer questions
- Why is the version read before the call? What goes wrong if it is read after, or at store time?
- Which lock do you hold while the backend runs? How do you coalesce without holding it?
- What exactly is in the in-flight map, who removes the entry, and what if the leader dies with an Error?
- A leader is interrupted or throws: what do the waiters see? Can a waiter wait forever?
- How do you avoid a thundering herd when a hot entry expires (V4 stale-while-revalidate)?
- `size()` is O(n) in a naive version. Is that acceptable? How would you make expiry cheaper (V5)?

## Invariants
- At most `capacity` entries; every stored entry has a version and a store time.
- For a normalised query there is at most one backend call in flight.
- An in-flight call that was invalidated never writes to the cache.
- A failure is delivered to every waiter of that call and never stored.
- Statistics change in the same critical section as the decision they describe.

## Common mistakes
- A single global lock held across the backend call: correct but serialises all queries (caught by the slow-call test).
- `check-then-act` race: two threads see a miss, both call the backend (coalescing missed), or both insert and the entry count exceeds capacity.
- Using `ConcurrentHashMap.computeIfAbsent` to run the backend: blocks the bucket, deadlocks on recursive updates, and cannot express the TTL/version check cleanly.
- Leaving a failed future in the in-flight map so every later call fails forever.
- Not removing the in-flight entry in a `finally` (an `Error` or interrupt leaves waiters hanging).
- Normalising for the key but passing the raw query to the backend, or normalising twice differently.
- Updating recency on a hit without synchronisation (an access-ordered `LinkedHashMap` is not safe for concurrent reads: `get` mutates it).
- Sliding TTL (refreshing the age on hit); off-by-one at the TTL boundary; stamping the version after the call.
- Counting `size()` with expired entries, or evicting a live entry while dead entries remain.
- Returning the backend's own list (later mutation leaks into the cache) or a mutable list that the caller can corrupt.

## Over-engineering traps
- A pluggable eviction/expiry policy hierarchy, a decorator stack and an event bus before LRU + TTL work.
- A background cleaner thread (nothing in the requirements needs one; it makes tests non-deterministic).
- A generic `Cache<K,V>` with loader plus a query-specific wrapper when one class would do.

## Design and scaling discussion
- Structures an interviewer hopes to hear: a hash map plus recency order (access-ordered linked hash map or map + doubly linked list) for O(1) LRU; an in-flight map from key to a shared future/promise (single-flight / request coalescing); immutable entry records `{results, storedAt, version}`; a sharded lock or striped locks if one lock becomes the bottleneck (shard by key hash, capacity per shard).
- Patterns that may come up: Proxy/Decorator (the cache in front of the backend interface), Strategy (eviction policy for the LFU follow-up, TTL provider for per-query TTL), Template Method/Loader (miss handling). Mention them only if justified.
- Version-based invalidation is O(1) (compare on read) compared with an explicit sweep on every index change; the cost is that dead entries occupy capacity until touched or evicted.
- Thundering herd vs cache stampede vs coalescing: coalescing deduplicates concurrent identical misses; stale-while-revalidate removes the latency spike; jittered TTLs spread expiries.
- Memory: bound by entry count here; real systems bound by bytes and cap the stored result size.

## Trade-off questions to ask when the candidate is done
- One lock or striped locks? What does the hit path cost under 64 threads?
- Should a waiter time out if the leader is slow? What would that change in the statistics?
- Would you cache failures briefly? Which failures? (V3)
- Why not publish index changes by pushing invalidations instead of polling the version?
- How would you test the coalescing without sleeping? (latch in the fake backend + `misses()` as a barrier)

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.

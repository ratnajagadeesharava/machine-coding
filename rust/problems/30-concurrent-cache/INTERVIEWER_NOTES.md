# Interviewer notes — 30 Concurrent Cache with Loaders

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- The ttl of a computed value starts when the loader finishes, not when it starts.
- A failed reload of an expired key leaves the key absent.
- A computed value inserted into a full cache evicts the least recently used entry.
- Concurrent callers after an expiry share one reload.
- Capacity 1.

## Likely interviewer questions
- Where does the loader run — inside a lock, or outside with a placeholder? What happens to waiters when it throws or when the computing thread is interrupted?
- How do you get LRU order with O(1) updates? What does a read cost when every read mutates the recency list?
- Do you purge expired entries lazily, with a sweeper thread, or both? What does `size()` cost?
- How would you avoid one global lock (striping, per-key futures, concurrent maps)? What does striping do to *global* LRU order?
- What if a `put` for the same key lands while a computation is in flight?

## Invariants
- `size() <= capacity` at all times, even under contention.
- At most one loader runs per key at a time.
- A failed computation leaves no trace (no value, no poisoned placeholder).
- Expired entries are never returned.

## Common mistakes
- Holding the cache lock while the loader runs (serialises unrelated keys, deadlocks with nested loaders).
- Check-then-act on `get` followed by `put` (many loaders run for one key).
- Leaving a failed placeholder in the map so later callers see the old failure forever.
- Wrapping the loader's exception so callers see a different type or message.
- Treating a read as not-a-use (broken LRU) or as extending the ttl.
- Evicting a live entry while an expired one is still occupying a slot.
- Off-by-one on expiry (`>` instead of `>=`).

## Over-engineering traps
- `EvictionPolicy` interface + `LruPolicy` + `CacheLoader` + `CacheBuilder` + listener hooks for a two-rule cache.
- A background sweeper thread when lazy purging on access satisfies every rule.

## Concurrency discussion
- Simple and correct: one lock for the map and recency order, held briefly; an in-flight table keyed by key holding a future/condvar; the loader runs with no lock held.
- Striped locks reduce contention but make LRU approximate — a conscious trade-off to state.
- Reads mutate recency, so a read lock is not enough unless recency is tracked separately.

## Trade-offs worth probing
- Exact global LRU vs approximate (sampled or per-stripe) LRU.
- Waiters blocking forever on a hung loader: timeouts, per-call deadlines.
- Lazy expiry vs timed sweeper (memory held by dead entries).

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.

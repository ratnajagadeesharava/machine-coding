# 30 · Concurrent Cache with Loaders (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 3 — infrastructure provided. Test utilities for threads/clocks exist; business/domain code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

**What you are building.** A service keeps the results of expensive lookups (a database row, a remote API answer) in
memory so it does not repeat the work. You build a small in-process cache for string keys and string values. It
holds a limited number of entries, forgets each entry after a fixed time, throws out the least recently used entry
when it is full, and can also *compute* a missing value on demand by calling a loader the caller supplies. When many
threads ask for the same missing key at once, the loader must run only once and everybody shares its result.

### Terms used below

- **ttl** ("time to live"): how many milliseconds an entry stays usable after it was stored.
- **Live** entry: stored and not yet expired. An **expired** entry is treated exactly as if it were not there.
- **Capacity**: the maximum number of live entries.
- **LRU (least recently used)**: the live entry whose last "use" is the oldest.
- **Loader**: a caller-supplied function that takes the key and produces the value (or fails).
- **Computation in progress**: a loader call that has started and not finished yet for some key.

## Requirements

1. The cache holds at most **capacity** entries and each entry lives for **ttl** milliseconds.
2. `get(key)` returns the value if the key is present and not expired; otherwise "nothing" (empty `Optional` / `None`).
3. `put(key, value)` stores (or overwrites) a value.
4. `get_or_compute(key, loader)` returns the cached value if there is one; otherwise it obtains the value by calling the caller-supplied loader (with the key as argument), stores it, and returns it.
5. When many threads ask for the **same missing key** at the same moment, the expensive loader must not run once per thread — everybody should receive one shared result. Asking for **different** keys must not make threads wait for each other's loaders (two loaders for two keys must be able to run at the same time).
6. If a loader fails, the failure is reported to the callers that were waiting for that computation and nothing is cached, so the next request tries again.
7. When the cache is full, making room for a new key removes the entry that was **least recently used**.
8. `size` reports how many entries are currently usable (live).

## Constraints

- Thread-safe; many threads, hot keys, slow loaders (milliseconds to seconds).
- Time comes from the clock given to the factory; never read the wall clock directly.
- Assume `capacity >= 1` and `ttl >= 1`; loaders never return null.
- A loader may itself call the cache (for other keys) without deadlocking.

## Examples

Time and expiry (ttl 1000 ms, clock reads 5000 when `put("a","1")` is called):

| Clock | Call | Result |
|---|---|---|
| 5999 | `get("a")` | `1` (alive, age 999) |
| 6000 | `get("a")` | nothing (expired, age 1000); `size()` = 0 |

Reads do not extend the ttl; overwriting restarts it (ttl 1000):

```
t=0    put(a,1)
t=600  get(a) → 1                     (the read does NOT extend the life)
t=1000 get(a) → nothing               (expired, age 1000)

t=0    put(a,1)
t=600  put(a,2)                       (overwrite: the ttl restarts, a now lives until t=1600)
t=1200 get(a) → 2
t=1600 get(a) → nothing
```

Least recently used (capacity 2):

```
put(a,1) put(b,2) get(a)          → 1      (a is now more recently used than b)
put(c,3)                          → b is evicted
get(b) → nothing    get(a) → 1    get(c) → 3    size → 2
put(a,9) with a, b present        → overwrite only; nothing is evicted
```

Expired entries never take space (capacity 2, ttl 1000):

```
t=0    put(a,1)            t=600  put(b,2)      get(a)   → a is most recently used, b the least
t=1000 a has expired (age 1000), b is alive (age 400)
t=1000 put(c,3)            → a's slot is reclaimed; b is NOT evicted. get(b)=2, get(c)=3, size=2
```

Computing and sharing:

```
get_or_compute("k", key -> "v:" + key)   → "v:k"   (loader ran once; get("k") → "v:k")
get_or_compute("k", anything)            → "v:k"   (hit; the second loader is never called)
8 threads call get_or_compute("k", slowLoader) at once → the loader runs once, all 8 get its value
loader throws "boom" → the computing caller and every waiting caller see "boom"; get("k") → nothing;
                       the next get_or_compute("k", ...) runs a fresh loader
```

## Edge cases

- A key whose entry expired is recomputed by `get_or_compute` (loader runs again, new value is stored with a fresh ttl).
- `get_or_compute` on a present key counts as a use for the LRU order; a miss, an expired lookup or a failed load does not.
- A computation in progress does not use up a slot, and does not show in `size()`, until its value is stored.
- `size()` never exceeds `capacity`, not even while many threads work at once.
- A loader that reads other keys through the same cache (`get`/`get_or_compute` on other keys) must work.

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- Time rule: an entry stored at time `t` is **expired when `now >= t + ttl`** (alive at `t + ttl - 1`). Expired entries behave exactly as if they were absent everywhere (`get`, `get_or_compute`, `size`, capacity).
- The store time of a `put` is the time of the call; the store time of a computed value is the time the loader **finishes**. Overwriting a key restarts its ttl. Reads never extend a ttl.
- "Use" for the least-recently-used order means a `get` hit, a `put`, or a `get_or_compute` (hit or store). A miss, an expired lookup, or a failed load is not a use.
- Capacity rule: inserting a key that is not currently present while the cache holds `capacity` live entries evicts the least recently used **live** entry. Expired entries never take space — they are dropped before any live entry is evicted. Overwriting an existing live key never evicts. Computations in progress do not occupy space until their value is stored.
- `size()` = number of live (non-expired) entries; it never exceeds `capacity`.
- `get_or_compute`: if the key is absent or expired and a computation for it is already running, the caller waits for it and receives its outcome (value or failure) without calling its own loader. Otherwise the caller's loader runs exactly once for that computation, outside any lock that would block other keys. The loader may itself call the cache for other keys.
- A loader failure is delivered **unchanged** (same exception / same error string) to the computing caller and to every caller that was waiting for that computation. Nothing is stored; the next call after the failure starts a new computation.

## Test entry point (Rust)

The tests drive your system through the thin, primitive-typed contract below (`src/contract.rs`). It is
**not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up
in `src/lib.rs`.

```rust
//! Thin test entry point. Your domain model lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

pub trait CacheApi: Send + Sync {
    /// The live value for the key, if any.
    fn get(&self, key: &str) -> Option<String>;

    fn put(&self, key: &str, value: &str);

    /// Cached value, or the result of the loader (computed once for concurrent callers of the same
    /// missing key). A loader `Err` is returned unchanged to every caller waiting for that computation.
    fn get_or_compute(
        &self,
        key: &str,
        loader: &(dyn Fn(&str) -> Result<String, String> + Sync),
    ) -> Result<String, String>;

    /// Number of live (non-expired) entries.
    fn size(&self) -> usize;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(
    _capacity: usize,
    _ttl_millis: u64,
    _clock: Arc<dyn Clock>,
) -> Option<Arc<dyn CacheApi>> {
    None
}
```

## Language notes

The trait is `&self` + `Send + Sync`. The loader is `&(dyn Fn(&str) -> Result<String, String> + Sync)`; an `Err(msg)` is returned unchanged to the computing caller and every waiter.

## Running

```bash
cargo test -p p30_concurrent_cache
STRICT=1 cargo test -p p30_concurrent_cache   # fail instead of skip while unimplemented
cargo test -p p30_concurrent_cache --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p30_concurrent_cache --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

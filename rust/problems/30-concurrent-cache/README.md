# 30 · Concurrent Cache with Loaders (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 3 — infrastructure provided. Test utilities for threads/clocks exist; business/domain code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

A service keeps expensive lookups in memory. Build a bounded, time-limited in-process cache for string keys and
string values that can also *compute* missing values on demand — safely and efficiently when many threads ask for
the same thing at once.

## Requirements

1. The cache holds at most **capacity** entries and each entry lives for **ttl** milliseconds.
2. `get` returns the value if the key is present and not expired.
3. `put` stores (or overwrites) a value.
4. `getOrCompute(key, loader)` returns the cached value if there is one; otherwise it obtains the value by calling the caller-supplied loader, stores it, and returns it.
5. When many threads ask for the **same missing key** at the same moment, the expensive loader must not run once per thread — everybody should receive one shared result. Asking for **different** keys must not make threads wait for each other's loaders.
6. If a loader fails, the failure is reported to the callers that were waiting for that computation and nothing is cached, so the next request tries again.
7. When the cache is full, making room for a new key removes the entry that was **least recently used**.
8. `size` reports how many entries are currently usable.

## Constraints

- Thread-safe; many threads, hot keys, slow loaders (milliseconds to seconds).
- A loader must never run while the cache holds a lock that blocks unrelated keys.
- Time comes from the `Clock` given to the factory; never read the wall clock directly.
- Assume `capacity >= 1` and `ttl >= 1`; loaders never return null.

## Examples

```
capacity 2, ttl 1000ms
put(a,1) put(b,2) get(a)          → 1
put(c,3)                          → b is evicted (a was used more recently)
8 threads: getOrCompute(k, slowLoader) → the loader runs once, all 8 get its value
loader throws → every waiting caller sees the failure; the following call runs a fresh loader
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- Time rule: an entry stored at time `t` is **expired when `now >= t + ttl`** (alive at `t + ttl - 1`). Expired entries behave exactly as if they were absent everywhere (`get`, `getOrCompute`, `size`, capacity).
- The store time of a `put` is the time of the call; the store time of a computed value is the time the loader **finishes**. Overwriting a key restarts its ttl. Reads never extend a ttl.
- "Use" for the least-recently-used order means a `get` hit, a `put`, or a `getOrCompute` (hit or store). A miss, an expired lookup, or a failed load is not a use.
- Capacity rule: inserting a key that is not currently present while the cache holds `capacity` live entries evicts the least recently used **live** entry. Expired entries never take space — they are dropped before any live entry is evicted. Overwriting an existing live key never evicts. Computations in progress do not occupy space until their value is stored.
- `size()` = number of live (non-expired) entries; it never exceeds `capacity`.
- `getOrCompute`: if the key is absent or expired and a computation for it is already running, the caller waits for it and receives its outcome (value or failure) without calling its own loader. Otherwise the caller's loader runs exactly once for that computation, outside any lock that would block other keys. The loader may itself call the cache for other keys.
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

The trait is `&self` + `Send + Sync`. The loader is `&(dyn Fn(&str) -> Result<String, String> + Sync)`; an `Err(msg)` must reach every waiter unchanged. Think about what you hold while the loader runs.

## Running

```bash
cargo test -p p30_concurrent_cache
STRICT=1 cargo test -p p30_concurrent_cache   # fail instead of skip while unimplemented
cargo test -p p30_concurrent_cache --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p30_concurrent_cache --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

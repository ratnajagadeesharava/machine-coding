# 74 · Concurrent LRU Cache (Read-Write Lock) (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 60 min | Level 3 — infrastructure provided. Test utilities for threads/clocks exist; business/domain code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Build a bounded least-recently-used cache that is shared by many threads. Most operations are reads, so you want
readers to run in parallel - but in an LRU cache a *read* (`get`) also reorders the recency list. Decide which operations
can truly share a lock and which cannot, keep the map and the recency order consistent at all times, and never exceed the
capacity, even transiently.

## Requirements

1. A cache has a fixed `capacity` (>= 1) and maps keys to values; keys and values are never null.
2. `put(key, value)` inserts or replaces; the key becomes the most recently used. If the cache is over capacity afterwards, the **least** recently used key is evicted.
3. `get(key)` returns the value and marks the key as most recently used.
4. `peek(key)` returns the value **without** changing the usage order. It is a pure read.
5. `remove(key)` deletes an entry and reports whether it was present.
6. `size()` is the number of entries and is never above the capacity.

## Constraints and thread-safety requirements

- Any number of threads call any operation at any time.
- Readers that only `peek` should be able to run in parallel with each other (that is the point of a read-write lock).
- `get` mutates the usage order, so it needs exclusive access to the order - make sure you do not corrupt it by calling it under a read lock.
- Never upgrade a held read lock to a write lock (it deadlocks with a standard read-write lock). Release, re-acquire, and re-check.
- The map and the usage order must never disagree: every key in one is in the other.

## Examples

```
capacity 2:  put(1) put(2)       -> usage order (old -> new): 1 2
             get(1)              -> 2 1
             put(3)              -> evicts 2 -> 1 3
             peek(1)             -> 1 3   (unchanged)
```

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `SharedCache<K, V>` (tests use `Integer` / `String`). Rust: `SharedCache` over `u64` keys and `String` values, `len()` instead of `size()`.
- `put` of an existing key replaces the value and refreshes the key; it never evicts anything else.
- Operations are linearizable: each takes effect at one instant between call and return.
- The tests only use values derived from the key (`"v" + key`), so a value returned for another key is detected.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

#[allow(clippy::len_without_is_empty)]
pub trait SharedCache: Send + Sync {
    /// Returns the value and marks the key as the most recently used.
    fn get(&self, key: u64) -> Option<String>;

    /// Returns the value WITHOUT changing the usage order (a pure read).
    fn peek(&self, key: u64) -> Option<String>;

    /// Inserts or replaces; the key becomes the most recently used. Evicts the least recently used key if over capacity.
    fn put(&self, key: u64, value: String);

    /// Returns true if the key was present.
    fn remove(&self, key: u64) -> bool;

    /// Number of entries; never above the capacity.
    fn len(&self) -> usize;
}
```

Your wiring point (`src/lib.rs`):

```rust
/// `capacity` is at least 1.
pub fn create(_capacity: usize) -> Option<Arc<dyn SharedCache>> { ... }
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Concurrency tests run many threads against one object, use deadlock timeouts (a hang becomes a failure) and repeat a workload 100 times to catch rare interleavings.

- Single-threaded semantics: LRU eviction, `get` refreshes, `peek` does not, replace refreshes, remove, capacity 1.
- 8 threads insert 8,000 distinct keys: nothing is lost.
- 12 threads of mixed put/get/peek/remove over 500 keys with capacity 50: `size()` is never above 50, no key ever returns another key's value, and at the end `size()` equals the number of keys really present.
- 8 threads x 5,000 new keys into capacity 50 end with exactly 50 entries.
- 16 threads `get` the same 100 keys 20,000 times each (the recency list is rewired concurrently): no misses, no corruption.
- A hot key touched by the only writer after every insert is never evicted while other threads read it.
- **Repeat 100x** a mixed workload with invariants checked after every run.

## Language notes

`RwLock<Inner>` where `Inner` owns the map and the order. Rust will not let `get(&self)` mutate through a read guard, which is the type system telling you what the design question is. An index-based list (`Vec` slots + prev/next indices) avoids `unsafe` and `Rc<RefCell>`.

## Running

```bash
cargo test -p p74_rw_lock_lru_cache
STRICT=1 cargo test -p p74_rw_lock_lru_cache   # fail instead of skip while unimplemented
cargo test -p p74_rw_lock_lru_cache --features hidden   # interviewer scenarios
cargo play run 74              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`run_concurrently`, `eventually`, `stays`, `watchdog`, `sut!`): `rust/support`. `run_concurrently` aborts the test run with a `TIMEOUT` message after 60 s, so a deadlock fails instead of freezing `cargo test`.

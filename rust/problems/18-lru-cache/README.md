# 18 · LRU Cache (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — minimal external API. The public interface is part of the problem; everything behind it is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Build a fixed-capacity in-memory cache. When it is full and a new entry arrives, the entry that has gone unused the
longest is thrown away to make room. Lookups and updates must stay fast no matter how large the cache is.

## Requirements

1. The cache maps keys to values and holds at most `capacity` entries; the capacity is fixed when the cache is created.
2. A lookup of a present key returns its value and counts as a **use** of that entry. A lookup of an absent key returns nothing and changes nothing.
3. Storing a key that is not present adds it. If that would exceed the capacity, the entry whose last use is the **oldest** is discarded first.
4. Storing a key that is already present replaces its value, counts as a use, and never discards anything.
5. An entry can be removed explicitly. Removing frees its slot immediately, so a later insert does not discard anything while there is room.
6. The current number of entries can be queried at any time.
7. The capacity must be at least 1.

## Constraints

- Up to hundreds of thousands of entries and millions of operations: `get`, `put` and `remove` must not scan the cache.
- Single-threaded use.

## Examples

```
capacity 2:  put(a,1)  put(b,2)  get(a) -> 1   put(c,3)   // b was used least recently, so b is discarded
             get(b) -> empty     get(a) -> 1    get(c) -> 3
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `get(key)` returns the value if present (and refreshes the entry), otherwise empty.
- `put(key, value)` inserts or replaces; afterwards the key is the most recently used entry.
- `remove(key)` returns `true` if an entry was removed, `false` if the key was absent.
- `size()` is the number of entries currently held, always `<= capacity`.
- Keys are compared with equality (`equals` / `Eq`); keys and values are never null.
- Creating a cache with capacity below 1 is invalid.

## Test entry point (Rust)

The tests drive your system through the thin, primitive-typed contract below (`src/contract.rs`). It is
**not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up
in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it. Single-threaded (`&mut self`).

#[allow(clippy::len_without_is_empty)]
pub trait Cache<K, V> {
    /// Present: returns a clone of the value and counts as a use. Absent: `None`, nothing changes.
    fn get(&mut self, key: &K) -> Option<V>;

    /// Insert or replace; the key becomes the most recently used. Evicts the least recently used entry on overflow.
    fn put(&mut self, key: K, value: V);

    /// `true` if an entry was removed.
    fn remove(&mut self, key: &K) -> bool;

    fn len(&self) -> usize;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(_capacity: usize) -> Option<Box<dyn Cache<String, i32>>> {
    None
}
```

## Language notes

The trait is `&mut self`; `get` returns a clone of the value. The factory fixes `K = String`, `V = i32`, but write your type generically if you like. Capacity below 1 is documented as invalid; the Rust tests do not exercise it (the only Java/Rust asymmetry), so decide how you want to signal it (panic or `None`). An arena with indices is the idiomatic way to build the recency list.

## Running

```bash
cargo test -p p18_lru_cache
STRICT=1 cargo test -p p18_lru_cache   # fail instead of skip while unimplemented
cargo test -p p18_lru_cache --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p18_lru_cache --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

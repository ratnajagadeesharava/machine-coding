# 19 · LFU Cache (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 60 min | Level 2 — minimal external API. The public interface is part of the problem; everything behind it is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Build a fixed-capacity in-memory cache that, when full, throws away the entry that has been used the **fewest times**.
Popular entries must survive a stream of one-off keys. Operations must stay fast however large the cache is.

## Requirements

1. The cache maps keys to values and holds at most `capacity` entries; the capacity is fixed when the cache is created.
2. Every entry has a **use count**. A newly inserted entry starts at 1.
3. A lookup of a present key returns its value and increases its use count by 1. A lookup of an absent key returns nothing and changes nothing.
4. Storing a key that is already present replaces its value and also increases its use count by 1. It never discards anything.
5. Storing a key that is not present adds it. If that would exceed the capacity, the entry with the **lowest use count** is discarded first. If several entries share the lowest count, the one that was **used least recently** is discarded.
6. An entry can be removed explicitly. Its use count is forgotten: if the key is stored again later it starts at 1 like any new key. The same holds for a key that was discarded by eviction.
7. The current number of entries can be queried at any time.
8. The capacity must be at least 1.

## Constraints

- Up to hundreds of thousands of entries and millions of operations: `get`, `put` and `remove` must not scan the cache.
- Single-threaded use.

## Examples

```
capacity 2:  put(1,A)  put(2,B)  get(1) -> A      // use counts: 1 -> 2, 2 -> 1
             put(3,C)                              // 2 has the lowest count, so 2 is discarded
             get(2) -> empty    get(3) -> C        // counts: 1 -> 2, 3 -> 2
             put(4,D)                              // tie at 2: key 1 was used less recently, so 1 is discarded
             get(1) -> empty    get(3) -> C    get(4) -> D
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `get(key)` returns the value if present (and counts as a use), otherwise empty.
- `put(key, value)` inserts or replaces. A replacement counts as a use; an insertion starts at count 1 and, when the cache was full, first discards one entry by the rule above. The newly inserted key is never the one discarded.
- "Used least recently" refers to the most recent `get` or `put` of that key (insertion counts as a use for recency).
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

    /// Insert (count 1) or replace (counts as a use). Evicts the least frequently, then least recently, used entry on overflow.
    fn put(&mut self, key: K, value: V);

    /// `true` if an entry was removed; its use count is forgotten.
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

The trait is `&mut self`; `get` returns a clone of the value. The factory fixes `K = String`, `V = i32`, but write your type generically if you like. Capacity below 1 is documented as invalid; the Rust tests do not exercise it (the only Java/Rust asymmetry), so decide how you want to signal it (panic or `None`). A `BTreeSet<(count, tick, key)>` is a perfectly reasonable first design; discuss when the bucket list is worth it.

## Running

```bash
cargo test -p p19_lfu_cache
STRICT=1 cargo test -p p19_lfu_cache   # fail instead of skip while unimplemented
cargo test -p p19_lfu_cache --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p19_lfu_cache --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

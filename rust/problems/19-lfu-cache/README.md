# 19 · LFU Cache (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 60 min | Level 2 — minimal external API. The public interface is part of the problem; everything behind it is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building a fixed-capacity in-memory cache that, when full, throws away the entry that has been used the
**fewest times** (least frequently used, "LFU"). Where an LRU cache forgets whatever was touched longest ago, an LFU
cache lets popular entries survive a long stream of one-off keys, because each popular entry has a high use count.
Operations must stay fast however large the cache is.

## Glossary

- **Capacity**: the maximum number of entries; fixed when the cache is created.
- **Use**: a successful `get`, or a `put` of the key (insert or replace).
- **Use count**: how many times an entry has been used since it was inserted. An insert counts as the first use, so a new entry has count 1.
- **Tie**: several entries share the lowest use count; the one used **least recently** (oldest last use) loses.
- **Evict**: discard an entry automatically to make room.

## Requirements

1. The cache maps keys to values and holds at most `capacity` entries; the capacity is fixed when the cache is created.
2. Every entry has a **use count**. A newly inserted entry starts at 1.
3. A lookup (`get`) of a present key returns its value and increases its use count by 1. A lookup of an absent key returns nothing and changes nothing.
4. Storing (`put`) a key that is already present replaces its value and also increases its use count by 1. It never discards anything (even when the cache is full).
5. Storing a key that is not present adds it. If that would exceed the capacity, the entry with the **lowest use count** is discarded first. If several entries share the lowest count, the one that was **used least recently** is discarded. The newly inserted key is never the one discarded.
6. An entry can be removed explicitly. Its use count is forgotten: if the key is stored again later it starts at 1 like any new key. The same holds for a key that was discarded by eviction.
7. The current number of entries can be queried at any time and is never above the capacity.
8. The capacity must be at least 1.

## Constraints

- Up to hundreds of thousands of entries and millions of operations: `get`, `put` and `remove` must not scan the cache.
- Single-threaded use.

## Examples

Count first, then recency as tie-break:

```
capacity 2:  put(1,A)  put(2,B)  get(1) -> A      // use counts: 1 -> 2, 2 -> 1
             put(3,C)                              // 2 has the lowest count, so 2 is discarded
             get(2) -> empty    get(3) -> C        // counts: 1 -> 2, 3 -> 2
             put(4,D)                              // tie at 2: key 1 was used less recently, so 1 is discarded
             get(1) -> empty    get(3) -> C    get(4) -> D
```

All new keys have count 1, so the oldest one goes:

```
capacity 2:  put(a,1)  put(b,2)  put(c,3)          // a and b tie at 1, a is older: a is discarded
```

A heavily used key survives; a fresh key is the next victim:

```
capacity 2:  put(a,1)  get(a)  get(a)  // a: count 3
             put(b,2)                  // b: count 1
             put(c,3)                  // b is discarded, not a; now c (count 1) is the next victim
```

Replacing counts as a use; removing forgets the count:

```
capacity 2:  put(a,1) put(b,2) put(a,10)   // a: count 2 (value now 10), b: count 1
             put(c,3)                       // b is discarded
capacity 2:  put(a,1) get(a) get(a)  remove(a) -> true   put(a,1)   // a is brand new again: count 1
             put(b,2)  put(c,3)             // a and b tie at 1; a is older: a is discarded
```

## Edge cases

- Capacity `1`: every insert of a new key replaces the only entry, whatever its count.
- A lookup of an absent key changes no counts and no recency.
- A `put` that replaces never evicts, even when the cache is full.
- Recency for tie-breaking is the time of the last `get` or `put` of the key (insertion counts).
- Creating a cache with capacity `0` or negative is invalid (see the language notes).

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

The trait is `&mut self`; `get` returns a clone of the value. The factory fixes `K = String`, `V = i32`, but write your type generically if you like. Capacity below 1 is documented as invalid; the Rust tests do not exercise it (the only Java/Rust asymmetry), so decide how you want to signal it (panic or `None`).

## Running

```bash
cargo test -p p19_lfu_cache
STRICT=1 cargo test -p p19_lfu_cache   # fail instead of skip while unimplemented
cargo test -p p19_lfu_cache --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p19_lfu_cache --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

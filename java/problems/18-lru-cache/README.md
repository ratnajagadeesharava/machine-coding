# 18 · LRU Cache (Java)

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

## Test entry point (Java)

The tests drive your system through the thin, primitive-typed contract below. It is **not** your domain
model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in
`solution/Solution.java`.

```java
package contract;

import java.util.Optional;

/** Thin test entry point. Your design lives behind it. Single-threaded. */
public interface Cache<K, V> {

    /** Present: returns the value and counts as a use. Absent: empty, nothing changes. */
    Optional<V> get(K key);

    /** Insert or replace; the key becomes the most recently used. Evicts the least recently used entry on overflow. */
    void put(K key, V value);

    /** True if an entry was removed. */
    boolean remove(K key);

    int size();
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    <K, V> Cache<K, V> create(int capacity)
}
```

## Language notes

`Cache<K,V>` is generic; `create` must throw `IllegalArgumentException` for a capacity below 1. `Optional` is used for `get`, so null values are never stored. `LinkedHashMap` is allowed in a real interview but ask yourself what you are then demonstrating.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/18-lru-cache test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

# 74 · Concurrent LRU Cache (Read-Write Lock) (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 60 min | Level 3 — infrastructure provided. Test utilities for threads/clocks exist; business/domain code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Build a bounded least-recently-used cache that is shared by many threads. Most operations are reads, so you want
readers to run in parallel - but in an LRU cache a *read* (`get`) also reorders the recency list. Keep the map and the recency order consistent at all times, and never exceed the
capacity, even transiently.

### What you are building

A small in-memory cache with a fixed size, like the hot-data cache in front of a database in a busy web service.
Many request threads store and fetch values at the same time. When the cache is full and a new key arrives, the entry
that has gone longest without being used is thrown away to make room. Lookups are far more common than inserts, so
the cache should let pure lookups run side by side.

### Glossary

- **Capacity**: the maximum number of entries; fixed when the cache is created.
- **Use**: a `put` or a `get` of a key. A `peek` is **not** a use.
- **Usage order**: the keys ordered from least recently used (LRU, next to be evicted) to most recently used (MRU).
- **Linearizable**: every operation appears to happen at one single instant between its call and its return.

## Requirements

1. A cache has a fixed `capacity` (>= 1) and maps keys to values; keys and values are never null.
2. `put(key, value)` inserts or replaces; the key becomes the most recently used. If the cache is over capacity afterwards, the **least** recently used key is evicted.
3. `get(key)` returns the value and marks the key as most recently used.
4. `peek(key)` returns the value **without** changing the usage order. It is a pure read.
5. `remove(key)` deletes an entry and reports whether it was present.
6. `size()` is the number of entries and is never above the capacity.
7. `get` or `peek` of a key that is not present returns `Optional.empty()` and changes nothing (a miss is not a use of anything). A new cache has size 0.
8. `put` of a key that is already present replaces the value, makes it the most recently used and does **not** change the size. `put` of a new key into a full cache evicts exactly one entry, the least recently used one, so the size stays equal to the capacity.
9. `remove` frees a slot (the size drops by one when the key was present) and does not change the relative order of the other keys. A removed key that is `put` again is a brand-new most-recently-used entry.

## Constraints and thread-safety requirements

- Any number of threads call any operation at any time.
- Readers that only `peek` should be able to run in parallel with each other.
- The map and the usage order must never disagree: every key in one is in the other.

## Examples

```
capacity 2:  put(1) put(2)       -> usage order (old -> new): 1 2
             get(1)              -> 2 1
             put(3)              -> evicts 2 -> 1 3
             peek(1)             -> 1 3   (unchanged)
```

More worked examples (single-threaded; usage order is written old -> new):

```
capacity 2:  put(1,"a") put(2,"b")   -> 1 2
             put(1,"a2")             -> 2 1      (replace refreshes; size stays 2)
             put(3,"c")              -> evicts 2 -> 1 3
             peek(2)                 -> empty

capacity 3:  put(1) put(2) put(3)    -> 1 2 3
             remove(2) -> true       -> 1 3      (size 2)
             remove(2) -> false
             put(4)                  -> 1 3 4    (no eviction: there was a free slot)

capacity 1:  put(1,"a") put(2,"b")   -> only key 2 remains; peek(1) is empty
```

## Edge cases

- Capacity 1: every new key evicts the previous one.
- `get` of the least recently used key rescues it from the next eviction; `peek` of it does not.
- Many threads `get` the same keys at once: no lookup of a key that stays present may report a miss, and nothing is corrupted.
- At no moment (not even inside `put`) may `size()` report more than the capacity.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `SharedCache<K, V>` (tests use `Integer` / `String`). Rust: `SharedCache` over `u64` keys and `String` values, `len()` instead of `size()`.
- `put` of an existing key replaces the value and refreshes the key; it never evicts anything else.
- Operations are linearizable: each takes effect at one instant between call and return.
- The tests only use values derived from the key (`"v" + key`), so a value returned for another key is detected.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.Optional;

/** Thin test entry point. Your design lives behind it. Must be thread-safe; keys and values are never null. */
public interface SharedCache<K, V> {

    /** Returns the value and marks the key as the most recently used. */
    Optional<V> get(K key);

    /** Returns the value WITHOUT changing the usage order (a pure read). */
    Optional<V> peek(K key);

    /** Inserts or replaces; the key becomes the most recently used. Evicts the least recently used key if over capacity. */
    void put(K key, V value);

    /** Returns true if the key was present. */
    boolean remove(K key);

    /** Number of entries; never above the capacity. */
    int size();
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    /**
 * Wiring point between the test contract and YOUR design.
 * Create your own classes anywhere under src/main/java and return them from here.
 */
public final class Solution {
    private Solution() {}

    /** {@code capacity} is at least 1. */
    public static <K, V> SharedCache<K, V> create(int capacity)
}
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

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/74-rw-lock-lru-cache test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`Concurrently`, `Eventually`, `Candidate`): `java/testkit`.

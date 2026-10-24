# 140 · Custom HashMap (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 60 min | Level 2 — contract (including the `KeyHasher` interface) and tests provided; the storage, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which state, which operations, which costs.
> Nothing here tells you how. See `INTERVIEW.md` for the process, `HINTS.md` if you are stuck
> (three levels, smallest nudge first) and `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

A map from `String` keys to `String` values that you implement yourself. **The intent is that you do not use any of the
language's built-in map or set types (`HashMap`, `ConcurrentHashMap`, `LinkedHashMap`, `TreeMap`, `Set`, ...) to store
the entries.** Arrays, your own classes and plain lists for *results* are fine. A black-box test cannot check this rule, so
the tests probe what a real hash table does: where keys end up when many of them collide, when and how the table grows,
the order in which entries are listed, how long things take, and that the hash function you were given is the only thing
used to place keys.

The hash function is supplied by the caller through a small provided interface, `KeyHasher`. The tests plug in a good
hasher, a hasher where *everything* collides, and hashers that return negative or extreme numbers. Your map must behave
correctly with all of them.

## Requirements

### Creation

`Solution.create(initialCapacity, loadFactorPercent, hasher)` returns a new empty map.

- `initialCapacity` must be between 1 and 1,048,576 inclusive; `loadFactorPercent` between 10 and 100 inclusive;
  `hasher` must not be null. Anything else is an `IllegalArgumentException`.
- Every key's position is decided with `hasher.hash(key)` only. The returned `int` may be any value, including negative
  numbers and `Integer.MIN_VALUE`, and different keys may return the same number.
- The hasher is called **exactly once** by every `put`, `putIfAbsent`, `get`, `getOrDefault`, `remove` and `containsKey`
  call that has valid arguments (also when the map is empty, when the key exists, when the call triggers a resize).
  It is **never** called by `size`, `isEmpty`, `capacity`, `keys`, `entries`, `clear`, and never for a call that is
  rejected because of a null argument.

### Operations

| Operation | Behaviour |
|---|---|
| `put(key, value)` | Stores the value. Returns the previous value of the key, or empty if the key was new. |
| `putIfAbsent(key, value)` | Stores only if the key is absent (returns empty). Otherwise changes nothing and returns the existing value. |
| `get(key)` | The value, or empty. |
| `getOrDefault(key, defaultValue)` | The value, or `defaultValue` when the key is absent. Stores nothing. |
| `remove(key)` | Removes the key and returns its previous value, or empty if it was absent. |
| `containsKey(key)` | Whether the key is present. |
| `size()` / `isEmpty()` | Number of entries / whether there are none. |
| `clear()` | Removes every entry. `capacity()` does not change. |
| `capacity()` | The current number of buckets (see below). |
| `keys()` | A new list of all keys in **insertion order**. |
| `entries()` | A new list of the strings `key + "=" + value`, in the same order as `keys()`. |

- Keys are compared by exact string equality, so they are case-sensitive. The empty string is a valid key and a valid
  value. Keys and values containing `=` are valid and are not escaped in `entries()`.
- A `null` key, value or default is an `IllegalArgumentException` and changes nothing.
- `keys()` and `entries()` return snapshots: later changes to the map do not alter a list that was already returned,
  and changing the returned list does not alter the map.

### Capacity

`capacity()` is the number of buckets of the table.

1. It starts at the smallest power of two that is `>= initialCapacity` (so an initial capacity of 1 gives 1, 3 gives 4,
   17 gives 32, 1000 gives 1024).
2. After every insertion of a **new** key (a `put` or `putIfAbsent` that adds an entry) the map checks the rule
   `size * 100 > capacity * loadFactorPercent`, where `size` already counts the new entry. If it holds, the capacity
   **doubles, once** (even if the rule would still hold afterwards: the next new key doubles again).
3. Replacing the value of an existing key, `putIfAbsent` on an existing key, `remove` and `clear` never change the
   capacity. The capacity never shrinks.

Capacity after N insertions of distinct keys, with `create(16, 75, ...)`:

| N (distinct keys inserted) | capacity |
|---:|---:|
| 1 – 12 | 16 |
| 13 – 24 | 32 |
| 25 – 48 | 64 |
| 49 – 96 | 128 |
| 97 – 192 | 256 |

With `create(1, 10, ...)` the first seven new keys each double the table (2, 4, 8, 16, 32, 64, 128) and the eighth does not.

### Order

- `keys()` lists keys in the order they were first inserted. Growing the table does not change this order.
- Putting a key that already exists replaces the value and **keeps its position**; `putIfAbsent` on an existing key
  changes nothing.
- A key that is removed and then put again is a new key and goes to the **end**.
- `clear()` forgets all order; after it the next key is first.

### Performance bounds

- With a good hasher, 1,000,000 `put`s followed by 1,000,000 `get`s must finish within 60 seconds, and
  `capacity()` must follow the rule above exactly (2,097,152 for the default parameters and one million keys).
- Removing keys must not cost time proportional to the size of the whole map: 200,000 removals in random order from a
  200,000-entry map must finish within 30 seconds (interviewer scenario).
- With a hasher for which **every key collides**, behaviour must remain correct. Quadratic time is accepted for this case
  up to this size only: 5,000 `put`s, 5,000 `get`s and 2,500 `remove`s must finish within 20 seconds.

### Concurrency

Every map returned by `create` is safe to use from many threads, and each single operation is atomic: it takes effect
as one indivisible step, so no put is lost, a `putIfAbsent` on one key has exactly one winner, and `keys()` / `entries()`
return a snapshot that no other operation is half-way through. There is no atomicity across two calls (a `get`
followed by a `put` can interleave with another thread). The hasher must be treated as a plain function that any
thread may call.

## Examples

```
m = create(16, 75, hasher)
m.put("a", "1")              -> empty
m.put("a", "2")              -> Optional["1"]
m.get("a")                   -> Optional["2"]
m.get("A")                   -> empty                (case-sensitive)
m.putIfAbsent("a", "x")      -> Optional["2"]        (unchanged)
m.putIfAbsent("b", "x")      -> empty                (stored)
m.getOrDefault("zzz", "d")   -> "d"                  (nothing stored)
m.remove("a")                -> Optional["2"]
m.keys()                     -> ["b"]
```

```
Order
put(a,1) put(b,2) put(c,3)   keys() -> [a, b, c]
put(a, again)                keys() -> [a, b, c]        (position kept)
remove(a); put(a, new)       keys() -> [b, c, a]        (removed then put: last)
entries()                    -> ["b=2", "c=3", "a=new"]
put("", "empty")             entries() ends with "=empty"
```

```
Capacity, create(4, 100, hasher)
put k0..k3 (4 keys)          capacity() -> 4            (4*100 > 4*100 is false)
re-put k0..k3, putIfAbsent   capacity() -> 4            (no new key)
put k4                       capacity() -> 8            (5*100 > 4*100)
remove all 5                 capacity() -> 8            (never shrinks)
put 8 keys                   capacity() -> 8
put a 9th                    capacity() -> 16
```

```
Hasher calls (counting hasher, create(2, 50, counting))
put("a","1"); put("a","2"); get("nope"); size(); keys()   -> hasher called 3 times in total
200 more puts of new keys (several resizes)              -> hasher called 200 more times, not more
```

## Edge cases

- Two different keys with the same hash number are two different keys. All of `get`, `remove`, `containsKey` and
  `put` must distinguish them (`"Aa"` and `"BB"` even have equal `String.hashCode()`).
- A hasher returning a negative number or `Integer.MIN_VALUE` must not break placement (no negative index, no
  exception).
- Growth depends on the number of entries only, never on how many keys collide: with an all-colliding hasher the
  capacity follows exactly the same table.
- Adding and removing the same few keys again and again does not grow the table (growth is only checked after an
  insertion of a new key, against the *current* size).
- `remove` of a missing key, `get` of a missing key and `clear` on an empty map are fine and change nothing.
- Invalid `create` arguments: `0`, `-1`, `1_048_577` capacity; `9`, `101` load factor; null hasher.

## Glossary

- **Bucket**: one slot of the table; `capacity()` counts them.
- **Load factor**: how full the table may get, as a percentage of its buckets, before it grows.
- **Collision**: two different keys that the hasher sends to the same place.
- **Resize**: replacing the table with a bigger one and moving the entries over.
- **Insertion order**: the order in which keys were first added (and not removed since).

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `IllegalArgumentException` for invalid creation arguments and for any null key, value or default.
- Rust: `create` returns `Err(MapError::InvalidArgument)` for invalid creation arguments; the Rust API takes `&str`, so there are no nulls.
- The hasher is called exactly once per `put`, `putIfAbsent`, `get`, `getOrDefault`, `remove`, `containsKey` and never otherwise.
- Capacity rule, order rules, snapshot rules and atomicity as above.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

/** Provided; implemented by the tests. Any int is a legal result. */
public interface KeyHasher {
    int hash(String key);
}

/** Every operation is atomic on its own. Null arguments: IllegalArgumentException. */
public interface StringMap {
    Optional<String> put(String key, String value);
    Optional<String> putIfAbsent(String key, String value);
    Optional<String> get(String key);
    String getOrDefault(String key, String defaultValue);
    Optional<String> remove(String key);
    boolean containsKey(String key);
    int size();
    boolean isEmpty();
    void clear();
    int capacity();
    List<String> keys();      // snapshot, insertion order
    List<String> entries();   // snapshot, "key=value", same order
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static StringMap create(int initialCapacity, int loadFactorPercent, KeyHasher hasher)
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. No test sleeps or
depends on the clock.

- Put/get/remove basics, previous values, case sensitivity, empty-string keys and values, `=` inside keys and values.
- Null arguments, invalid creation parameters, and that rejected calls never reach the hasher.
- Initial capacity rounding, the default-parameter capacity table, other load factors (10%, 100% and a model check), growth only on new keys, never shrinking, `clear` keeping the capacity.
- Insertion order across resizes, re-put keeping the position, remove-then-put going last, `entries()` format, snapshots.
- `putIfAbsent` and `getOrDefault`.
- A counting hasher: exactly one call per keyed operation (including the one that resizes), none for the others.
- Negative and extreme hash values; keys that collide; 5,000 keys with an all-colliding hasher; one million puts and gets.
- Concurrency: 8 threads putting disjoint keys lose nothing.
- More scenarios (200,000 removals, concurrent `putIfAbsent`, consistent snapshots under contention, churn that must not grow the table) are in `interviewer/`.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/140-custom-hashmap test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`Candidate`, `Concurrently`): `java/testkit`.

# 21 · In-Memory Key-Value Store (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 90 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Build an in-memory key-value store like a tiny Redis: values can expire after a time-to-live, keys can be listed by
prefix, and callers can group several changes into a transaction that they may later keep or throw away. Transactions
can be nested.

## Requirements

1. Keys and values are strings. `put` stores or overwrites a value; `get` returns it (or nothing); `delete` removes a key.
2. `putWithTtl` stores a value that **expires**: a key whose value was set at time `t` with a TTL of `ttl` milliseconds is gone once `now >= t + ttl`. The TTL must be positive.
3. A plain `put` on a key replaces its value **and** removes any TTL it had. Calling `putWithTtl` again restarts the countdown from the moment of that call.
4. An expired key behaves exactly like a key that was never stored: `get` finds nothing, `delete` reports that nothing was deleted, it is not listed and not counted.
5. `keysWithPrefix` returns the live keys that start with the prefix, in ascending order. The empty prefix matches every key.
6. `size` is the number of live keys.
7. `begin` opens a transaction. Transactions nest: `begin` inside a transaction opens a child of it.
8. Inside a transaction every operation (reads included) sees that transaction's own writes and deletes on top of what its parents (and finally the store) hold.
9. `rollback` discards the changes made since the innermost open `begin` and closes it. `commit` closes the innermost transaction and keeps its changes: they are merged into the parent transaction, or into the store itself when it was the outermost one. Changes merged into a parent are still subject to the parent's own commit or rollback.
10. `commit` and `rollback` with no open transaction do nothing and report failure.
11. The moment a TTL starts is the moment of the `putWithTtl` call, not the moment of a later commit. Time keeps running regardless of transactions: whatever is expired at the time of a read is invisible, whichever transaction level holds it. Rolling back restores previous values with their original deadlines.

## Constraints

- Up to 100,000 keys; `keysWithPrefix` and `size` are called often and should not be slower than they need to be.
- Time comes from the `Clock` given to the factory.
- Single-threaded use.

## Examples

```
now=0:    putWithTtl("session", "abc", 1000)       get at 999 -> "abc"      get at 1000 -> nothing
put("a","1"); begin(); put("a","2"); delete("b"); get("a") -> "2"; rollback() -> true; get("a") -> "1"
begin(); put("x","1"); begin(); put("x","2"); commit(); rollback(); get("x") -> nothing   (the inner commit joined the outer transaction)
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `get` returns empty for unknown, deleted or expired keys. `delete` returns `true` only if a live key was removed.
- `putWithTtl` returns `false` and changes nothing when `ttlMillis <= 0`; otherwise `true`. `put` has no return value.
- `keysWithPrefix` compares keys by plain character order (tests use ASCII keys), returns a fresh list and never includes expired keys.
- `size` counts live keys as seen from the innermost open transaction.
- `begin()` always succeeds. `commit()` / `rollback()` return `true` when a transaction was closed and `false` when none was open.

## Test entry point (Java)

The tests drive your system through the thin, primitive-typed contract below. It is **not** your domain
model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in
`solution/Solution.java`.

```java
package contract;

import java.util.List;
import java.util.Optional;

/** Thin test entry point. Your domain model lives behind it. */
public interface KeyValueStoreApi {

    void put(String key, String value);

    /** Refused (false, no change) when ttlMillis <= 0. Expired once now >= setAt + ttlMillis. */
    boolean putWithTtl(String key, String value, long ttlMillis);

    Optional<String> get(String key);

    /** True only if a live key was removed. */
    boolean delete(String key);

    /** Live keys starting with the prefix, ascending. Empty prefix = all. */
    List<String> keysWithPrefix(String prefix);

    /** Number of live keys. */
    int size();

    void begin();

    /** Keeps the innermost transaction's changes; false if none is open. */
    boolean commit();

    /** Discards the innermost transaction's changes; false if none is open. */
    boolean rollback();
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    KeyValueStoreApi create(Clock clock)
}
```

## Language notes

`Clock` is `java.time.Clock`; read `clock.millis()`. Single-threaded: no locking needed.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/21-key-value-store test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

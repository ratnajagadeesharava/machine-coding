# 21 · In-Memory Key-Value Store (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 90 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building an in-memory key-value store, like a tiny Redis. Callers store string values under string keys.
Values can **expire** after a time-to-live (a login session that lasts 30 minutes), keys can be listed by prefix
(`user:` -> all user keys), and callers can group several changes into a **transaction** that they may later keep or
throw away, like a database. Transactions can be nested, and time keeps running while they are open.

## Glossary

- **Live key**: a key that currently has a value: it was stored, not deleted, and (if it has a TTL) not yet expired.
- **TTL (time-to-live)**: how many milliseconds a value stays alive after it was stored with `putWithTtl`.
- **Deadline**: the instant the key expires: `time of the putWithTtl call + ttl`. At that instant the key is already gone.
- **Transaction**: a group of changes that can be committed (kept) or rolled back (discarded) together.
- **Nested transaction**: `begin` while another transaction is open; the new one is the child and is the **innermost** one.
- **Parent**: the enclosing transaction, or the store itself for the outermost transaction.

## Requirements

1. Keys and values are strings. `put` stores or overwrites a value; `get` returns it (or nothing); `delete` removes a key.
2. `putWithTtl` stores a value that **expires**: a key whose value was set at time `t` with a TTL of `ttl` milliseconds is gone once `now >= t + ttl` (alive at `t + ttl − 1`). The TTL must be positive.
3. A plain `put` on a key replaces its value **and** removes any TTL it had. Calling `putWithTtl` again restarts the countdown from the moment of that call.
4. An expired key behaves exactly like a key that was never stored: `get` finds nothing, `delete` reports that nothing was deleted, it is not listed and not counted.
5. `keysWithPrefix` returns the live keys that start with the prefix, in ascending order (plain character order, so `user:10` comes before `user:2`). The empty prefix matches every key.
6. `size` is the number of live keys.
7. `begin` opens a transaction. Transactions nest: `begin` inside a transaction opens a child of it.
8. Inside a transaction every operation (reads included) sees that transaction's own writes and deletes on top of what its parents (and finally the store) hold.
9. `rollback` discards the changes made since the innermost open `begin` and closes it. `commit` closes the innermost transaction and keeps its changes: they are merged into the parent transaction, or into the store itself when it was the outermost one. Changes merged into a parent are still subject to the parent's own commit or rollback.
10. `commit` and `rollback` with no open transaction do nothing and report failure (`false`).
11. The moment a TTL starts is the moment of the `putWithTtl` call, not the moment of a later commit. Time keeps running regardless of transactions: whatever is expired at the time of a read is invisible, whichever transaction level holds it. Rolling back restores previous values with their original deadlines. A rollback never brings back a key whose deadline has passed.

## Constraints

- Up to 100,000 keys; `keysWithPrefix` and `size` are called often and should not be slower than they need to be.
- Time comes from the `Clock` given to the factory.
- Single-threaded use.

## Examples

TTL (clock starts at time 0):

```
putWithTtl("session", "abc", 1000) -> true
at 999:  get("session") -> "abc"          at 1000: get("session") -> nothing
```

Restarting and dropping a TTL:

```
t=0  putWithTtl("k","v1",1000)    t=600 putWithTtl("k","v2",1000)    -> deadline is now 1600
t=1599 get("k") -> "v2"           t=1600 get("k") -> nothing
putWithTtl("k","v",100); put("k","v2")   -> no TTL any more; still there at t=10000
```

Rollback and nesting:

```
put("a","1"); begin(); put("a","2"); delete("b"); get("a") -> "2"; rollback() -> true; get("a") -> "1"

begin(); put("x","1"); begin(); put("x","2"); commit() -> true;  get("x") -> "2"
rollback() -> true;  get("x") -> nothing        (the inner commit only joined the outer transaction)

begin(); begin(); put("x","deep"); commit(); commit(); get("x") -> "deep"; rollback() -> false
```

TTL and transactions:

```
t=0:   begin(); putWithTtl("k","v",1000)      t=600: commit()      (deadline stays 1000, not 1600)
t=999: get("k") -> "v"                         t=1000: get("k") -> nothing

t=0:   putWithTtl("k","v1",1000)   t=500: begin(); put("k","v2"); rollback()
t=999: get("k") -> "v1"            t=1000: get("k") -> nothing    (original deadline restored)

t=0:   putWithTtl("k","v",100); begin(); delete("k"); t=200: rollback()
get("k") -> nothing, size() -> 0     (the rollback does not resurrect an expired key)
```

Listing and counting inside a transaction (store has a=1, b=2):

```
begin(); put("a","changed"); delete("b"); put("c","3")
get("b") -> nothing; keysWithPrefix("") -> [a, c]; size() -> 2; delete("b") -> false
```

## Edge cases

- `putWithTtl` with `ttl <= 0` returns `false` and changes nothing (the key keeps its old value and TTL, or stays absent).
- Overwriting an expired key with `put` or `putWithTtl` simply creates it afresh.
- A key that expires while a transaction is open disappears from `get`, `keysWithPrefix` and `size` immediately, even though the transaction still holds it.
- `delete` of a key already deleted inside the current transaction returns `false`.
- After the outermost `commit`, a further `rollback` returns `false` (the changes are permanent).
- `begin` always succeeds and returns nothing.

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

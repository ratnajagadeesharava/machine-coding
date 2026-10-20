# 103 · CRDT Replicated Store (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 75 min | Level 3 — infrastructure provided: a deterministic network and clock simulation (`SimNet`) runs your nodes; the protocol code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Our collaboration app keeps counters (likes), sets (tags) and single values (document title) on replicas in several regions. Every replica must accept writes on its own, even when it cannot reach the others, and when replicas later exchange their state they must all end up showing the same values, without a coordinator deciding who wins and without losing concurrent updates that should survive.

Build one replica of such a store. Replicas exchange whole-state snapshots as opaque strings and `merge` them, in any order, any number of times, possibly duplicated or stale. The store has three data types, each with its own conflict rule.

## Requirements

1. PN-counter: `increment` / `decrement` by a non-negative amount (a negative amount is rejected); the value is the sum of all increments minus the sum of all decrements from all replicas. An untouched counter is 0. Concurrent updates on different replicas must not be lost.
2. OR-set (observed-remove, add wins): `add`, `remove`, `contains`, `elements`. `remove` removes only the additions this replica has OBSERVED; an add concurrent with a remove survives the merge. An element can be re-added after being removed. Removing an absent element does nothing. `elements` returns a copy.
3. LWW-register: `write(value, timestamp)`; the highest timestamp wins; equal timestamps are decided by the larger replica id (string comparison); a write older than the current value is ignored. A never-written register is empty.
4. The three types live in separate namespaces: a counter, a set and a register may share the same name without interfering.
5. `snapshot()` returns the whole state as an opaque string that is a VALUE: later changes to the replica never alter an already taken snapshot. `merge(snapshot)` folds another replica's state into this one.
6. `merge` is idempotent, commutative and associative, never loses and never resurrects anything: merging an old snapshot must not undo newer state, merging yourself or an empty replica is harmless.
7. The replica is thread-safe: many threads may update it concurrently without losing updates.

## Simulation rules and constraints

- No network simulation here: replicas are plain in-process objects; the tests move snapshots between them by hand, in any order, duplicated, stale or reordered (that is the simulated network).
- The replica must be thread-safe: the tests hammer one replica from 8 threads at once (counter increments, set adds, register writes) and require that no update is lost. Java: any thread; Rust: the trait takes `&self`, so use a `Mutex` inside.
- Determinism: there is no wall clock, no randomness required. Timestamps are supplied by the caller and a replica never writes the same timestamp twice to one register.
- `snapshot()` and `merge()` can be called concurrently with updates and with each other; a snapshot must be a consistent state, not a half-updated one.
- Snapshots must stay compact: the counter state must not grow with the number of increments, and merging 20,000 set elements (half removed) must be fast.

## Examples

```
a.increment("views",5)   b.increment("views",3)  b.decrement("views",1)   -> a.merge(b.snap), b.merge(a.snap): both read 7
a.add("s","x")  sync(a,b)   then  a.remove("s","x") || b.add("s","x")   -> after sync x is still in the set (add wins)
a.write("r","from-a",10)  b.write("r","from-b",10)                         -> after sync both read "from-b" (larger id)
```

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `Solution.replica(replicaId)`; `increment/decrement(String, long)` throw `IllegalArgumentException` for a negative amount. Rust: `create(replica_id)`; amounts and timestamps are `u64` (so a negative amount cannot be expressed), `counter -> i64`, `elements -> HashSet<String>`, `read -> Option<String>`.
- Counter value = (sum of increments over all replicas) - (sum of decrements over all replicas); each replica's contribution is kept per replica and merged by maximum, so merging twice or merging an old snapshot cannot double-count or go backwards.
- OR-set: each addition gets a unique identity; a remove records exactly the additions observed locally. An element is present iff it has an addition that no merged remove observed. Re-adding after a remove creates a new addition that old removes (even delivered later) must not hide.
- A stale snapshot that still contains a removed element must not bring it back (the remove knowledge is part of the state and merged by union).
- LWW-register: compare (timestamp, replica id) pairs; the larger wins. Equal timestamps from different replicas are resolved by the lexicographically larger replica id. Values and names may contain newlines, separators, quotes and Unicode and must round-trip through `snapshot` / `merge`.
- Order-independence: merging the same set of snapshots in any order, repeated any number of times, gives identical counter, set and register values.
- The snapshot format is yours; it only needs to be understood by your own `merge`.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.Optional;
import java.util.Set;

/**
 * One replica of a small replicated data store made of state-based CRDTs. Every replica accepts writes on its own
 * (no coordination); replicas exchange whole-state snapshots and {@link #merge(String)} them, in any order, any number
 * of times, possibly duplicated. After the replicas have merged each other's state they must all show the same values.
 *
 * <p>Three data types, each living in its own namespace (a counter "a", a set "a" and a register "a" are unrelated):
 * <ul>
 *   <li><b>PN-counter</b>: {@code increment}/{@code decrement}, value = sum of all increments - sum of all decrements
 *       from all replicas. A counter nobody touched is 0. A negative amount is an {@link IllegalArgumentException}.</li>
 *   <li><b>OR-set</b> (observed-remove, add wins): {@code remove} only removes the additions this replica has
 *       OBSERVED. An add that is concurrent with a remove survives the merge. An element can be re-added after a
 *       remove. Removing an element that is not in the set does nothing.</li>
 *   <li><b>LWW-register</b>: {@code write(value, timestamp)}: the highest timestamp wins; equal timestamps are broken
 *       by the larger replica id (string comparison); a write with an older timestamp than the current value is
 *       ignored. A replica never writes the same timestamp twice to one register (the tests do not do it). Never written: empty.</li>
 * </ul>
 *
 * <p>The snapshot is an opaque string only your own {@code merge} has to understand. It must be a VALUE: later changes
 * of this replica must not change a snapshot that was already taken. {@code merge} must be idempotent, commutative and
 * associative and must never lose or resurrect anything: merging an old snapshot does not undo newer state.
 * The replica must be thread-safe.
 */
public interface Replica {

    String id();

    void increment(String counter, long amount);

    void decrement(String counter, long amount);

    long counter(String counter);

    void add(String set, String element);

    void remove(String set, String element);

    boolean contains(String set, String element);

    /** A copy: later changes do not show up in it. */
    Set<String> elements(String set);

    void write(String register, String value, long timestamp);

    Optional<String> read(String register);

    String snapshot();

    void merge(String snapshot);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static Replica replica(String replicaId)
}
```

## What the tests check

These tests are black-box and deterministic: every node runs inside a seeded simulation (one tick = 1 ms of simulated time, nothing sleeps), messages can be delayed, dropped, duplicated and reordered, nodes can crash and recover, the network can be partitioned, and safety invariants are checked after EVERY simulated millisecond. A failing seed reproduces exactly.

- Local counter semantics: starts at 0, increments and decrements, other names unaffected; negative amounts rejected (Java).
- Concurrent increments on two replicas are summed after a two-way sync; merging the same snapshot repeatedly does not double-count; merging an old snapshot after a newer one does not roll back.
- Snapshots are values: later local changes do not leak into a snapshot already taken.
- OR-set: add / remove / re-add; `elements` is a copy; removals replicate; a concurrent add survives a remove; a remove only affects what it observed (an unobserved add on another replica survives).
- No resurrection: a stale snapshot merged after a remove does not bring the element back; a re-added element survives an old remove delivered again.
- LWW-register: highest timestamp wins locally, older writes ignored, three replicas converge under different merge orders, equal timestamps go to the larger replica id.
- Namespaces: a counter, a set and a register with the same name are independent; awkward names and values (newlines, separators, quotes, Unicode) round-trip.
- Merging an empty replica and merging oneself are harmless.
- 8 threads x 1,000 operations on one replica lose nothing; 100 random schedules of mixed operations and stale, duplicated, reordered snapshot deliveries converge to the expected counter and identical sets and registers on three replicas.
- More edge cases (any merge order of four replicas' snapshots, snapshot size growth, 20,000-element sets) in `interviewer/` (see `INTERVIEWER_NOTES.md`).

## Language notes

Escape or length-prefix the strings in your snapshot (names can contain newlines and separators). `Collectors`/`TreeMap` give deterministic output; copy under a lock (or `synchronized`) so `snapshot` is consistent.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/103-crdt-replication test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`SimNet`, `FakeClock`, `Candidate`): `java/testkit`.

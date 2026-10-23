# 103 · CRDT Replicated Store (Rust)

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

**In short:** one replica object holding named counters, sets and registers. It must accept updates alone, produce a `snapshot()` string, and `merge(snapshot)` strings from other replicas so that all replicas eventually show identical values, whatever the order, duplication or staleness of the snapshots.

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
- The replica must be thread-safe: the tests hammer one replica from 8 threads at once (counter increments, set adds, register writes) and require that no update is lost. Java: any thread; Rust: the trait takes `&self`.
- Determinism: there is no wall clock, no randomness required. Timestamps are supplied by the caller and a replica never writes the same timestamp twice to one register.
- `snapshot()` and `merge()` can be called concurrently with updates and with each other; a snapshot must be a consistent state, not a half-updated one.
- Snapshots must stay compact: the counter state must not grow with the number of increments, and merging 20,000 set elements (half removed) must be fast.

## Examples

```
a.increment("views",5)   b.increment("views",3)  b.decrement("views",1)   -> a.merge(b.snap), b.merge(a.snap): both read 7
a.add("s","x")  sync(a,b)   then  a.remove("s","x") || b.add("s","x")   -> after sync x is still in the set (add wins)
a.write("r","from-a",10)  b.write("r","from-b",10)                         -> after sync both read "from-b" (larger id)
```

(`sync(a, b)` = take both snapshots first, then each replica merges the other's.)

Counters never double-count and never go backwards:

```
a.increment("c",1);  old = a.snapshot();  a.increment("c",4)
b.merge(a.snapshot())      -> b reads 5
b.merge(old)               -> b still reads 5 (old state is a subset of what b already has)
b.merge(a.snapshot())      -> b still reads 5 (merging the same state again changes nothing)
a.decrement("c",9)         -> a reads -4; negative values are fine
```

Observed remove:

```
a.add("s","x");  b.remove("s","x")  (b has never seen a's add, so there is nothing for b to remove)
sync(a,b)                           -> x is in the set on both

a.add("s","x"); stale = a.snapshot(); b.merge(stale); b.remove("s","x"); a.merge(b.snapshot())   -> x is gone on a and b
b.merge(stale); a.merge(stale)      -> x stays gone: the removal is part of the state, an old snapshot cannot resurrect it

a.add("s","x"); a.remove("s","x"); afterRemove = a.snapshot(); a.add("s","x"); b.merge(a.snapshot())
b.merge(afterRemove)                -> x is present: the re-add is a new addition that the old remove never observed
```

Register with three writers:

```
a.write("r","from-a",5)   b.write("r","from-b",9)   c.write("r","from-c",7)
any merge order, any repetition  -> every replica reads "from-b" (highest timestamp)
a.write("r","older",3) on a replica that holds timestamp 5 or more -> ignored
```

Snapshots are values:

```
a.increment("c",1); snap = a.snapshot(); a.increment("c",100); a.add("s","x")
b.merge(snap)   -> b.counter("c") == 1 and b.contains("s","x") == false
```

## Edge cases

- Amounts and timestamps are unsigned (`u64`), so a negative amount cannot be expressed; an amount of 0 is allowed and changes nothing.
- Counters, sets and registers live in separate namespaces: `counter("x")`, `elements("x")` and `read("x")` are three independent things, even if all three exist in the same replica and travel in the same snapshot.
- A counter nobody touched reads 0; a register nobody wrote reads empty; a set nobody touched has no elements. `remove` of an element that is not present does nothing.
- `elements` returns a copy: adding to the set afterwards does not change a set you already received.
- Names and values may contain newlines, `|`, `;`, `,`, quotes and non-ASCII characters (for example `"line1\nline2 | with ;,'\" and ünïcode ✓"`) and must survive `snapshot` followed by `merge` unchanged.
- Merging an empty replica's snapshot, or your own snapshot, is harmless. Snapshots may be merged in any order and any number of times; the result is the same.
- A replica never writes the same timestamp twice to one register, but different replicas can use the same timestamp: the larger replica id wins (compared as strings: `"b" > "a"`).
- Concurrent use: 8 threads each doing 1,000 increments, adds and writes on one replica must lose nothing (the counter reads 8,000), and `snapshot` / `merge` may run at the same time as updates.
- Size: the counter part of a snapshot must not grow with the NUMBER of increments, and merging a snapshot with 20,000 set elements (half of them removed) must be fast.

## Glossary

- **Replica**: one copy of the data store, with a unique id, that accepts writes on its own.
- **State-based CRDT**: a data type whose replicas exchange their whole state and merge it; merging is idempotent (repeat is harmless), commutative (order does not matter) and associative (grouping does not matter).
- **PN-counter**: a counter supporting both increments and decrements by keeping the two totals per replica.
- **OR-set (observed-remove set)**: a set where `remove` only deletes the additions this replica has seen; an add that happens concurrently survives ("add wins").
- **LWW-register (last-writer-wins)**: one value; the write with the highest (timestamp, replica id) pair wins.
- **Concurrent**: two updates on different replicas where neither replica had seen the other's update.
- **Stale snapshot**: an old snapshot delivered late; it must not undo anything newer.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `Solution.replica(replicaId)`; `increment/decrement(String, long)` throw `IllegalArgumentException` for a negative amount. Rust: `create(replica_id)`; amounts and timestamps are `u64` (so a negative amount cannot be expressed), `counter -> i64`, `elements -> HashSet<String>`, `read -> Option<String>`.
- Counter value = (sum of increments over all replicas) - (sum of decrements over all replicas); each replica's contribution is kept per replica and merged by maximum, so merging twice or merging an old snapshot cannot double-count or go backwards.
- OR-set: each addition gets a unique identity; a remove records exactly the additions observed locally. An element is present iff it has an addition that no merged remove observed. Re-adding after a remove creates a new addition that old removes (even delivered later) must not hide.
- A stale snapshot that still contains a removed element must not bring it back (the remove knowledge is part of the state and merged by union).
- LWW-register: compare (timestamp, replica id) pairs; the larger wins. Equal timestamps from different replicas are resolved by the lexicographically larger replica id. Values and names may contain newlines, separators, quotes and Unicode and must round-trip through `snapshot` / `merge`.
- Order-independence: merging the same set of snapshots in any order, repeated any number of times, gives identical counter, set and register values.
- The snapshot format is yours; it only needs to be understood by your own `merge`.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point for the CRDT replica. One replica of a small replicated store made of state-based CRDTs.
//! Every replica accepts writes on its own (no coordination); replicas exchange whole-state snapshots and `merge`
//! them, in any order, any number of times, possibly duplicated. After the replicas have merged each other's state
//! they must all show the same values.
//!
//! Three data types, each living in its own namespace (a counter "a", a set "a" and a register "a" are unrelated):
//! - **PN-counter**: `increment`/`decrement`; value = sum of all increments - sum of all decrements from all
//!   replicas. A counter nobody touched is 0.
//! - **OR-set** (observed-remove, add wins): `remove` only removes the additions this replica has OBSERVED. An add
//!   that is concurrent with a remove survives the merge. An element can be re-added after a remove. Removing an
//!   element that is not in the set does nothing.
//! - **LWW-register**: `write(value, timestamp)`: the highest timestamp wins; equal timestamps are broken by the
//!   larger replica id (string comparison); a write older than the current value is ignored. A replica never writes
//!   the same timestamp twice to one register (the tests do not do it). Never written: `None`.
//!
//! The snapshot is an opaque string only your own `merge` has to understand. It must be a VALUE: later changes of
//! this replica must not change a snapshot that was already taken. `merge` must be idempotent, commutative and
//! associative and must never lose or resurrect anything: merging an old snapshot does not undo newer state.
//! The replica must be thread-safe (the trait takes `&self`).

use std::collections::HashSet;

pub trait Replica: Send + Sync {
    fn id(&self) -> String;

    fn increment(&self, counter: &str, amount: u64);

    fn decrement(&self, counter: &str, amount: u64);

    fn counter(&self, counter: &str) -> i64;

    fn add(&self, set: &str, element: &str);

    fn remove(&self, set: &str, element: &str);

    fn contains(&self, set: &str, element: &str) -> bool;

    /// A copy: later changes do not show up in it.
    fn elements(&self, set: &str) -> HashSet<String>;

    fn write(&self, register: &str, value: &str, timestamp: u64);

    fn read(&self, register: &str) -> Option<String>;

    fn snapshot(&self) -> String;

    fn merge(&self, snapshot: &str);
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(_replica_id: &str) -> Option<Arc<dyn Replica>> { ... }
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

The trait takes `&self`. Snapshot output is deterministic.

## Running

```bash
cargo test -p p103_crdt_replication
STRICT=1 cargo test -p p103_crdt_replication   # fail instead of skip while unimplemented
cargo test -p p103_crdt_replication --features hidden   # interviewer scenarios
cargo play run 103              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`SimNet`, `Rng`, `sut!`): `rust/support`.

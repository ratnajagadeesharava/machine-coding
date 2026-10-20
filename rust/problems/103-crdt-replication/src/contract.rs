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
//! The replica must be thread-safe (the trait takes `&self`: use a `Mutex`).

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

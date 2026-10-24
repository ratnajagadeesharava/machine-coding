# 122 · Versioned Value Store (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 75 min | Level 2 — the entry point and the tests are provided; the store itself, its data model and its concurrency story are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

A key-value store that never forgets by accident. Every successful write gets the next number in one store-wide count (1, 2, 3, ...), and the store can answer not only "what is the value of `k` now?" but also "what was the value of `k` when the store was at number 7?". On top of that you build three things that people expect from such a store: a **compare-and-set** write ("only if nobody has changed this key since I read it"), **snapshots** (a frozen view of the whole store that later writes cannot disturb), and **compaction** (throwing away old history to free memory, with clear rules about what can still be read afterwards). Many threads use the store at the same time, and the numbers they get back must still be unique, gap-free and consistent with what readers see.

## Problem

Keys and values are strings. The store keeps a global **version**: the number of successful writes so far. A write is a `put`, a `putAll` (a batch that counts as ONE write), a successful `delete`, or a successful `putIfVersion`. The store can be read "as of" any version it still retains. Deleting a key does not erase it: it records a **tombstone**, which is itself a versioned write.

## Requirements

1. **Versions.** `currentVersion()` is 0 for a store that was never written. Every successful write returns a new version equal to `currentVersion()` before it plus one. Versions are shared by all keys, strictly increasing and gap-free. A call that fails or changes nothing (invalid input, mismatching compare-and-set, deleting a key without a live value) consumes no version.
2. **`put(key, value)`** writes the value and returns its version. Writing the same value again is still a write with a new version. The empty string is a valid value.
3. **`putAll(entries)`** writes every entry under ONE new version and returns it. Any reader, `getAt` or snapshot, sees either all entries of a batch or none of them. An empty batch, or a batch containing an invalid key or value, is rejected as a whole: nothing is applied and no version is consumed.
4. **`delete(key)`** writes a tombstone and returns its version. If the key has no live value (never written, or already deleted) it returns -1 (Java) / `None` (Rust) and changes nothing.
5. **`get(key)`** returns the latest value, or empty if the key was never written or its latest write is a tombstone.
6. **`getAt(key, version)`** returns the value as of that version: the newest retained write of the key with version <= `version`; empty if there is none or it is a tombstone. A version below 1 or above `currentVersion()` is an error; a version below the compaction floor is a different error ("compacted", see 11).
7. **`history(key)`** returns the retained writes of the key, oldest first, as strings in this exact format: `<version>:<value>`, and `<version>:<deleted>` for a tombstone (a value is everything after the first colon; it may be empty). An unknown key has an empty history.
8. **`putIfVersion(key, value, expectedKeyVersion)`** compares `expectedKeyVersion` with the **key version**: the version of the key's current live value, or 0 if the key has no live value (never written, or deleted). If equal it behaves like `put` and returns the new store-wide version; otherwise it returns empty, changes nothing and consumes no version. A negative expected version is an error.
9. **`snapshot()`** opens a read view fixed at `currentVersion()` at that moment and returns an id (a positive number, different for every snapshot). `snapshotGet(id, key)` returns the key's value as of the snapshot's version and never sees later writes. `releaseSnapshot(id)` closes it. An unknown or already released id is an error for both.
10. **Invalid input.** A null or blank key is an error everywhere a key is taken. A null value is an error (Java; Rust values cannot be null). Errors leave the store unchanged.
11. **`compact(keepVersionsAtOrAfter)`** removes history that no read can need any more and returns the number of removed entries. Let *F* be the highest value passed to `compact` so far (the **floor**, initially 1). After a call, these reads must give exactly the same answers as before: `get`, `getAt` for every version >= *F*, and every open snapshot. `getAt` for a version below *F* fails with the "compacted" error (checked after the range check of requirement 6). The argument must satisfy 1 <= `keepVersionsAtOrAfter` <= `currentVersion()`, otherwise it is an error; a value lower than the current floor is accepted and removes nothing new.

   To make the removed count exact, the rule is: an entry of a key is **kept** if its version is >= *F*, or it is the newest entry of that key with version <= *F*, or it is the newest entry of that key with version <= the version of some open snapshot. Every other entry is removed. After that, if the oldest remaining entry of a key is a tombstone, it is removed as well, and a key left with no entries is forgotten (its history is empty and a later write simply starts it again). Versions keep counting from where they were.

## Examples

### Worked trace 1: versions, tombstones, time travel

```
call                    returns   store afterwards (version: key=value)
put(a, x)               1         1: a=x
put(b, y)               2         2: b=y
put(a, z)               3         3: a=z
delete(b)               4         4: b=<deleted>
delete(b)               -1        (no live value, nothing changes)
put(c, w)               5         5: c=w

currentVersion()  -> 5            get(b)  -> empty           get(a) -> z
getAt(a, 2) -> x                  getAt(b, 3) -> y           getAt(b, 4) -> empty
getAt(c, 4) -> empty              getAt(a, 0), getAt(a, 6) -> error
history(a) -> ["1:x", "3:z"]      history(b) -> ["2:y", "4:<deleted>"]
```

### Worked trace 2: compare-and-set

```
put(k, a)               1         key version of k is 1
putIfVersion(k, b, 0)   empty     (k has a live value; 0 does not match)
putIfVersion(k, b, 1)   2         key version of k is now 2
putIfVersion(k, c, 1)   empty     (stale; no version consumed)
delete(k)               3         k has no live value: its key version is 0 again
putIfVersion(k, d, 2)   empty     (2 was the version of the old value)
putIfVersion(k, d, 0)   4
currentVersion()        4
```

### Worked trace 3: snapshots and a batch

```
put(a, 1)               1
s = snapshot()          (fixed at version 1)
putAll({a: 2, b: 2})    2         ONE version for both keys
snapshotGet(s, a) -> 1            snapshotGet(s, b) -> empty        get(a) -> 2
releaseSnapshot(s)
snapshotGet(s, a)       -> error  (released)
```

### Worked trace 4: compaction

```
put(a, v1) 1   put(a, v2) 2   put(b, x) 3   put(a, v3) 4   delete(b) 5   put(c, z) 6

compact(4)  -> 2          a: entries 1, 2 removed (entry 4 is the newest <= 4); b: entry 3 is the newest <= 4, kept
history(a) -> ["4:v3"]    history(b) -> ["3:x", "5:<deleted>"]
getAt(b, 4) -> x          getAt(a, 3) -> error "compacted"
compact(4)  -> 0          compact(7) -> error (above currentVersion)

With a snapshot s opened at version 2 before the writes of versions 3..6, compact(4) would also keep a's entry 2
(the newest <= 2) and return 1 (only a's entry 1 is removed).
```

## Edge cases

- A failed or no-op call never consumes a version, even under concurrency: after N successful writes `currentVersion()` is exactly N.
- `putAll` with the same key listed twice is impossible in Java (a `Map`); in Rust it is an error.
- `putIfVersion(key, value, 0)` on a deleted key succeeds; `putIfVersion` with the tombstone's version fails.
- A tombstone written for a key that has no live value does not exist: `delete` returns -1 / `None` instead.
- `getAt` for a key that was never written, at a valid version, is empty (not an error). Below the floor it is still the "compacted" error.
- A snapshot taken before a compaction stays fully readable after it, even for versions below the floor; `getAt` does not (it only knows about the floor).
- A snapshot taken on an empty store (version 0) sees nothing, ever.
- `compact` on a store with no writes is an error (there is no valid argument).
- Two writes with the same key and the same value are two history entries.

## Glossary

- **Version**: the store-wide write counter; also the name of the point in time a write happened at.
- **Key version**: the version of a key's current live value, 0 if it has none.
- **Tombstone**: the record a successful delete leaves; it hides all older values of the key from reads at or after it.
- **As of version *v***: the state after the write with version *v* and before the next one.
- **Snapshot**: a read view pinned at the version the store had when it was opened, until it is released.
- **Compaction floor**: the highest argument passed to `compact` so far; `getAt` below it is refused.
- **Compare-and-set**: write only if the key is still at the version the caller last saw.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Rust: `Result<_, StoreError>` with `InvalidArgument` (blank key, empty batch, a key twice in a batch, a `get_at` version outside `1..=current_version()`, a `compact` argument outside `1..=current_version()`), `UnknownSnapshot` (unknown or released id) and `Compacted` (`get_at` below the compaction floor). Versions and snapshot ids are `u64`.
- `delete` returns `Ok(None)` when there is no live value; `put_if_version` returns `Ok(None)` on a mismatch; reads return `Result<Option<String>, _>`.
- Version numbers: 1, 2, 3, ... across all keys, gap-free. Every thread gets different version numbers; after N successful writes (including one per `putAll` batch) `currentVersion()` is N.
- Concurrent `putIfVersion` calls on one key with the same expected version: exactly one succeeds. Concurrent `delete` of one live key: exactly one gets a version.
- A snapshot taken while threads call `putAll` on a pair of keys shows both new values or both old values of every batch, never a mix.
- `history` format is exactly `<version>:<value>` / `<version>:<deleted>`.
- `compact` removes exactly the entries described in requirement 11 and returns their count.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Create whatever structs, enums and traits you like and wire them up in `create` in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

#[derive(Debug, PartialEq, Eq)]
pub enum StoreError {
    /// blank key, empty batch / duplicate key in a batch, version outside 1..=current, compaction floor outside
    /// 1..=current
    InvalidArgument,
    /// the snapshot id was never issued or was already released
    UnknownSnapshot,
    /// `get_at` for a version below the compaction floor
    Compacted,
}

pub trait VersionedStore: Send + Sync {
    /// Writes `value` for `key` and returns the new store-wide version (1, 2, 3, ... without gaps).
    /// A blank key is `InvalidArgument` (no version is consumed).
    fn put(&self, key: &str, value: &str) -> Result<u64, StoreError>;

    /// Writes all entries as ONE version and returns it: a reader sees all of them or none of them.
    /// An empty batch, a blank key or a key listed twice is `InvalidArgument`; nothing is applied and no version
    /// is consumed.
    fn put_all(&self, entries: &[(&str, &str)]) -> Result<u64, StoreError>;

    /// Deletes the key by writing a tombstone and returns `Some(version)`, or `None` if the key has no live value
    /// (never written, or already deleted); in that case nothing changes and no version is consumed.
    fn delete(&self, key: &str) -> Result<Option<u64>, StoreError>;

    /// The latest value, `None` if the key is deleted or was never written.
    fn get(&self, key: &str) -> Result<Option<String>, StoreError>;

    /// The value of `key` as of `version`: the newest write with version <= `version` (`None` if there is none or
    /// it is a tombstone). `InvalidArgument` if `version` is 0 or above `current_version()`; `Compacted` if it is
    /// below the compaction floor (see `compact`).
    fn get_at(&self, key: &str, version: u64) -> Result<Option<String>, StoreError>;

    /// Version of the latest successful write; 0 for a store that has never been written.
    fn current_version(&self) -> u64;

    /// Retained history of the key, oldest first, one string per entry: `"<version>:<value>"`, and
    /// `"<version>:<deleted>"` for a tombstone. Empty for an unknown key.
    fn history(&self, key: &str) -> Result<Vec<String>, StoreError>;

    /// Compare-and-set. Succeeds only if `expected_key_version` equals the version of the key's current live value,
    /// where 0 means "the key has no live value" (never written, or deleted). On success returns
    /// `Some(new store-wide version)`; on mismatch returns `None`, changes nothing and consumes no version.
    fn put_if_version(
        &self,
        key: &str,
        value: &str,
        expected_key_version: u64,
    ) -> Result<Option<u64>, StoreError>;

    /// Opens a read view fixed at `current_version()` right now and returns its id (a positive number, unique).
    fn snapshot(&self) -> u64;

    /// The value of the key in that snapshot's view. `UnknownSnapshot` for an unknown or released id.
    fn snapshot_get(&self, snapshot_id: u64, key: &str) -> Result<Option<String>, StoreError>;

    /// Closes the snapshot. `UnknownSnapshot` for an unknown or already released id.
    fn release_snapshot(&self, snapshot_id: u64) -> Result<(), StoreError>;

    /// Drops history that no read can need any more and returns how many entries were removed. Reads that stay
    /// answerable: `get_at` for every version >= `keep_versions_at_or_after`, `get`, and every open snapshot.
    /// Afterwards `get_at` for a version below the floor (the highest value passed so far) is `Compacted`.
    /// `InvalidArgument` unless 1 <= `keep_versions_at_or_after` <= `current_version()`.
    fn compact(&self, keep_versions_at_or_after: u64) -> Result<u64, StoreError>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create() -> Option<Arc<dyn VersionedStore>>
```

## What the tests check

These tests are black-box: they only call the contract above, so any design behind it works. No test sleeps or depends on timing. More scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Version numbering: start at 0, one counter for all keys, same value written twice, deleted keys, no version consumed by failed calls.
- Reads: latest value, `getAt` before the first write / at a tombstone / out of range, `history` format and order.
- `putAll` as one version, rejected as a whole when invalid; compare-and-set including after delete; invalid input everywhere.
- Snapshots: frozen view, independent views, distinct ids, unknown and released ids.
- Compaction: removed counts, floor behaviour, the entry visible at the floor, forgotten keys, open snapshots, range checks, floor never lowered.
- Concurrency: 8 threads writing distinct keys use every version exactly once; 8 threads on one key keep a strictly increasing history; 16 threads racing a compare-and-set or a delete have one winner; retrying compare-and-set loses no increment; snapshots taken during `putAll` batches are never torn; a snapshot stays stable under writers.
- More (a mixed workload checked against `history`, compaction racing writers and snapshots, tombstones pinned by snapshots, lock-style compare-and-set) is in `interviewer/`.

## Running

```bash
cargo test                          # from this directory
STRICT=1 cargo test                # fail instead of skip while unimplemented
cargo test --features hidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`run_concurrently`, `collect_concurrently`, `sut!`): `rust/support`.

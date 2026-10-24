# Interviewer notes — 122 Versioned Value Store

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Hundreds of thousands of puts and gets interleaved on 500 keys: versions unique and gap-free, every key's history complete.
- A mixed workload of put, delete and compare-and-set(0) on 20 keys from 8 threads: for every key and every version, `getAt` must agree with what `history` says. This catches a version assigned outside the lock that appends the entry (entries out of order in a key's list).
- Compaction running while writers do `putAll` and readers open and release snapshots: no torn pair, a long-lived pinned snapshot never changes.
- Snapshot pinning layers: two snapshots at different versions keep different entries alive; the entry disappears only when the last snapshot needing it is released.
- A tombstone must survive compaction when an older entry is kept for another snapshot (dropping it would resurrect the old value for the newer snapshot).
- Lock-style compare-and-set with expected 0 combined with delete: wins equal deletes, the history has exactly two entries per cycle.
- Overlapping `putAll` batches on different pairs of keys; each version appears on exactly two keys with the same value.

## Likely interviewer questions
- Where is the version number assigned, and what guarantees that the history of a key is sorted by version?
- What does a reader do while a writer is in the middle of a `putAll`? Why can it never see half a batch?
- What does `putIfVersion` compare with after a delete, and why did you choose that?
- What does opening a snapshot cost? Does it copy anything? What if nobody releases it?
- What is the complexity of `getAt` and of `compact`? Can compaction run without blocking readers?
- Why is "the newest entry at or before the floor" kept? What breaks if you drop it?
- What do you return for `getAt` of a key that was compacted away entirely?

## Invariants
- The store version equals the number of successful writes, and every retained entry has a version <= it.
- Within one key, entries are strictly increasing in version; a `putAll` puts one entry with the same version in each of its keys.
- A snapshot's answers never change after it is opened, regardless of writes and compaction.
- After `compact(F)`, every read at version >= F and every open snapshot gives the same answer as before.
- A failed, rejected or no-op call changes nothing and leaves no gap in the version numbers.

## Common mistakes
- Incrementing a counter with a separate atomic and appending to the key's list later: two threads append out of order, or a reader sees version N before N-1 is visible.
- Taking the version before validating, so an invalid call burns a number (gap).
- Making `putAll` a loop of `put`s: versions are consumed per key and readers see half a batch.
- `putIfVersion` that compares with the store version, or that reads the key version and writes in two separate steps.
- `getAt` off by one: `<` instead of `<=`, or returning the first entry after the version.
- Tombstone treated as a value (returns "<deleted>") or ignored (older value leaks through).
- Snapshot implemented by copying the whole map: correct but O(n) and memory-hungry; or by remembering only the version while compaction removes what it needs.
- Compaction that drops the entry visible at the floor, so `getAt(F)` becomes empty for keys not written since.
- Compaction that deletes a tombstone while an older entry is kept for a snapshot.
- Checking the compaction floor before the range check (or the other way round) so the error type differs from the README.
- Holding one global lock for reads too: correct but slow; or a lock per key with a global counter and no ordering between them.

## Over-engineering traps
- Persisting to disk, a write-ahead log or a distributed protocol as part of the core.
- A generic MVCC framework with transaction objects, isolation levels and pluggable storage engines before `put/get/getAt` work.
- A background compaction thread with scheduling policies; the contract compacts on demand.
- Persistent (immutable, structurally shared) maps for everything when a per-key list does the job — worth discussing, not required.

## Design and scaling discussion
- Per-key version lists plus one global counter is the simplest model: `getAt` is a binary search, `put` appends. The hard part is making "take version, append, publish" one atomic step across keys; a single write lock gives it for free, striping gives it with more care.
- Copy-on-write persistent maps make snapshots O(1) (hold a root) and readers lock-free, at the cost of allocation per write; per-key lists with a snapshot version need compaction to respect open snapshots explicitly.
- Read-write locks, a lock per key plus an atomic counter, or lock-free skip lists (`ConcurrentSkipListMap` keyed by (key, version)) each trade simplicity for write concurrency; ask the candidate which they chose and what ordering bug the others invite.
- MVCC databases keep the oldest open snapshot as the garbage-collection horizon; an abandoned snapshot is a memory leak, which is why real systems time them out.
- Compaction can be incremental per key or per segment so it does not stop the world; it needs a floor that only moves forward.
- A hybrid logical clock or a version-to-timestamp index supports time travel by timestamp without scanning.
- Transactions (V5) reuse the same version order: validate that the keys read are unchanged since the read version, then apply as one `putAll`-style version.

## Trade-off questions to ask when the candidate is done
- Global lock vs. per-key locks: what does each cost for `putAll` and for snapshots?
- Would you store the full value for every version, or deltas? What changes for `getAt` and compaction?
- What would you do about a snapshot that is never released?
- How would you test gap-freeness and torn batches without relying on luck? (The tests here use many threads and invariants that any interleaving must satisfy.)
- What is the blast radius of a compaction bug, and how would you verify it on production-sized data?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.

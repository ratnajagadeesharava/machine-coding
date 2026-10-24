# Rubric — 122 Versioned Value Store (Rust)

| Area | Weight |
|---|---:|
| Functional correctness | 40% |
| Design quality (model, API, responsibilities) | 15% |
| Code quality | 10% |
| Testing | 5% |
| Concurrency (declared scope, no shared mutable surprises) | 20% |
| Performance | 10% |

Score each area 0–4 (0 = absent, 2 = acceptable, 4 = would hire on this alone), multiply by weight.

## Questions for every problem

**Design**
- Did the candidate identify the correct domain concepts?
- Are responsibilities well separated; does each piece of state have one clear owner?
- Are abstractions justified by a requirement? (Not: "did they use pattern X?")
- Is the API clean, hard to misuse, and free of leaked internals?
- Can the requirements evolve (see `FOLLOWUPS.md`) without a rewrite?
- Is the implementation unnecessarily complicated?

**Correctness & quality**
- Are invariants preserved across every operation, including failed ones (no half-applied changes)?
- Are edge cases (empty, boundary, repeated, invalid operations) handled deliberately?
- Can components be tested independently of the whole system?

**Concurrency & performance**
- Is concurrency safe (or explicitly and correctly declared out of scope)? Is the locking granularity reasoned about?
- Are performance characteristics reasonable for the stated constraints?

**Language-specific**
- Are traits/enums/structs justified, or did you reach for a trait by reflex?
- Is the lock choice (`Mutex`, `RwLock`, atomics) deliberate, and are guards kept short and never held across calls into unknown code?
- Is mutable state encapsulated behind `&self` methods; are value types plain data?

## Problem-specific

- Is the version a store-wide, gap-free counter, and is it assigned in the same atomic step that makes the write visible (not before, not after)?
- Is `putAll` one version and one atomic step for readers, including snapshots taken concurrently?
- Is `getAt` fast (not a scan of all history), and is the "tombstone hides older values" rule correct at the boundaries?
- Is the key version for `putIfVersion` defined consistently after a delete, and is the check-and-write one atomic step?
- Do failed calls (invalid input, CAS mismatch, delete of a missing key) leave the version counter untouched?
- Does compaction keep exactly what the rules say (floor, snapshots, tombstones) and give the right count? Does a reader never see a half-compacted key?
- Is the cost of a snapshot reasonable (does opening one copy the store)? What does an unreleased snapshot cost?
- Did the candidate state a clear locking or immutability story and defend it with the concurrency tests?

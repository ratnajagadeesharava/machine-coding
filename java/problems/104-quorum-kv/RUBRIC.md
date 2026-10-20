# Rubric — 104 Quorum Key-Value Store (Java)

| Area | Weight |
|---|---:|
| Functional correctness | 30% |
| Design quality (model, API, responsibilities) | 10% |
| Code quality | 10% |
| Testing | 10% |
| Distributed reasoning (failures, ordering, retries, idempotency) | 35% |
| Performance | 5% |

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
- Are classes/interfaces justified, or did you reach for an interface by reflex?
- Are collections, locks and executors chosen on purpose (not `synchronized` everywhere)?
- Is mutable state encapsulated; are value types immutable (records / final fields)?

## Problem-specific

- Are quorum acks and answers counted per distinct node and compared against the full cluster majority?
- Is read repair performed, including for late answers, and is a stale write still acknowledged?
- Are the 100 ms / 1000 ms timers driven by `onTick` with FAILED only after the deadline and PENDING before?
- Are request ids and duplicated messages handled idempotently?
- Does the candidate know what the design does not guarantee (linearizability, lost concurrent writes, clock skew)?

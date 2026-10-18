# Rubric — 21 In-Memory Key-Value Store (Java)

| Area | Weight |
|---|---:|
| Functional correctness | 35% |
| Design quality (model, API, responsibilities) | 25% |
| Code quality | 15% |
| Testing | 15% |
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
- Are classes/interfaces justified, or did you reach for an interface by reflex?
- Are collections, locks and executors chosen on purpose (not `synchronized` everywhere)?
- Is mutable state encapsulated; are value types immutable (records / final fields)?

## Problem-specific

- Is the transaction model (layers or undo log) clean, with tombstones handled correctly?
- Are deadlines absolute and expiry evaluated consistently on every read path (get, delete, keys, size)?
- Is `keysWithPrefix` sorted and served without a full sort per call?
- Is the design free of speculative abstractions?

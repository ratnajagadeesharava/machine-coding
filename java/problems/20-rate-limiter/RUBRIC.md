# Rubric — 20 Rate Limiter (Java)

| Area | Weight |
|---|---:|
| Functional correctness | 25% |
| Design quality (model, API, responsibilities) | 15% |
| Code quality | 10% |
| Testing | 10% |
| Concurrency | 25% |
| Performance | 15% |

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

- Is the window rule implemented exactly (half-open interval, denied requests not recorded)?
- Is the check-and-record atomic per key, with a justified lock granularity?
- Is each decision O(1) amortised, and is stale per-key state pruned so memory is bounded?
- Does the design use the injected clock only and handle edge cases (time zero, tiny windows)?

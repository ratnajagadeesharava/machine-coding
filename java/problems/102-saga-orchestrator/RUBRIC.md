# Rubric — 102 Saga Orchestrator (Java)

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

- Is the progress log designed so both `status` and resumption are derived from the same records, and are write points justified?
- Are the retry/backoff series exactly as specified, including 'no sleep after the last attempt' and 'stop on COMPENSATION_FAILED'?
- Is mutual exclusion per saga (not global), with RUNNING returned to a second caller?
- Does the candidate state the idempotency assumption and the at-least-once consequence of a crash between execute and log?
- Is the crash-at-every-record property argued, not just tested on the happy path?

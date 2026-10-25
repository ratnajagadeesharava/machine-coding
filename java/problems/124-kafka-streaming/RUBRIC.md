# Rubric — 124 Kafka-Like Message Streaming Service (Java)

| Area | Weight |
|---|---:|
| Functional correctness | 40% |
| Design quality (model, API, responsibilities) | 20% |
| Code quality | 10% |
| Testing | 10% |
| Concurrency (declared scope, no shared mutable surprises) | 15% |
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

- Is the partition choice exactly the specified hash with wrap-around and a non-negative modulo, and is round-robin separate from keyed routing?
- Are offsets gap-free and never renumbered by retention; is `earliest`/`latest` consistent after drops?
- Are committed offsets owned by the group (not by the consumer or the assignment), and does polling leave them untouched?
- Is the range assignment a deterministic function of the sorted membership, recomputed on every join and leave?
- Do commit checks (membership, ownership, monotonic, not past the log end) all hold, and does a failed call leave no trace?
- Is publish atomic per partition (offset allocation, append, trim) and is a poll a consistent snapshot, without one global lock for every operation?

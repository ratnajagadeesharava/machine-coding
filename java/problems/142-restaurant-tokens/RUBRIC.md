# Rubric — 142 Restaurant Customer Tokens (Java)

| Area | Weight |
|---|---:|
| Functional correctness | 45% |
| Design quality (model, API, responsibilities) | 20% |
| Code quality | 10% |
| Testing | 10% |
| Concurrency (declared scope, no shared mutable surprises) | 5% |
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
- Are collections and locks chosen on purpose (not `synchronized` everywhere)? Is mutable state encapsulated; are value types immutable (records / final fields)?

## Problem-specific

- Is there exactly one serving order, used consistently by `callNext`, `queueSnapshot` and `positionInQueue`?
- Is "passed over" implemented exactly as specified (later-issued token called while the token waits, any counter), with promotion at 5 and oldest-first among promoted tokens?
- Is the token state machine explicit, with every illegal transition rejected and leaving state unchanged (including a refused recall)?
- Does a counter skip unsupported tokens without consuming or reordering them?
- Are grace / recall boundaries (`>` vs `>=`) and the time stamp of an expiry-created no-show right, using only the injected clock?
- Is the average an integer floor mean over served tokens only, and are numbers gap-free under concurrency?

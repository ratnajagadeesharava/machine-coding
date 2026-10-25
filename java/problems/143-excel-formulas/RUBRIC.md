# Rubric — 143 Excel Cell Values and Sum Formulas (Java)

| Area | Weight |
|---|---:|
| Functional correctness | 40% |
| Design quality (model, API, responsibilities) | 20% |
| Code quality | 10% |
| Testing | 10% |
| Concurrency (declared scope, no shared mutable surprises) | 5% |
| Performance and robustness at scale | 15% |

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

- Is the formula text turned into something evaluable once per `set`, not re-read from the string on every recomputation?
- Is every dependent re-evaluated exactly once per change, after everything it reads, including through diamonds and ranges? Is the counter exact?
- Are cycles detected for self-references, pairs, ranges and long rings, and do the cells hanging off a cycle behave as specified? Can the cycle be repaired?
- Does the 2,600-cell chain (and the 2,600-cell ring) work without recursion depth problems?
- Are old references removed when a formula is replaced or cleared, so unrelated changes recompute nothing?
- Are the error rules (syntax vs reference vs division vs text, left-to-right, first error in row order) exactly as specified?
- Is number formatting deterministic (rounding, `-0`, no exponent)?
- Do readers always see a consistent sheet, and is `set` atomic with respect to `get`?

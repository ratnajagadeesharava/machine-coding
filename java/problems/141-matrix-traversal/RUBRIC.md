# Rubric — 141 Matrix Traversal Strategies (Java)

| Area | Weight |
|---|---:|
| Functional correctness | 40% |
| Design quality (model, API, responsibilities) | 20% |
| Code quality | 10% |
| Testing | 10% |
| Concurrency (declared scope, no shared mutable surprises) | 5% |
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

- Do all nine orders come out right on non-square shapes and on 1 x N / N x 1, not just on a square?
- Is every built-in order a function of the visit index (so a cursor holds almost no state and `kthValue` does not walk), or does something quietly materialise the matrix or the sequence?
- Is the laziness real: `start` plus ten `next` calls, `pathSum(k)` and `kthValue` stay within the stated read budgets on the 20,000 x 20,000 source?
- Is arithmetic done in 64 bits wherever `rows*cols`, an index or a sum is involved?
- Is a registered strategy validated at the point of use, with the offending index named, without corrupting the cursor?
- Do cursor ids follow the contract (start at 1, never reused, closed means gone), and is a cursor shared between threads handled consistently?

# Rubric — 129 Job Posting with Candidate Matching (Java)

| Area | Weight |
|---|---:|
| Functional correctness | 40% |
| Design quality (model, API, responsibilities) | 20% |
| Code quality | 10% |
| Testing | 10% |
| Concurrency (declared scope, no shared mutable surprises) | 10% |
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

- Is eligibility (hard rules) kept apart from the score (soft preferences), with 0 meaning "not eligible" and every eligible pair at 50 or more?
- Is the score computed exactly as specified, including the caps, the `2 * salary` comparison without overflow, and the skill listed in both sets?
- Are skills and locations normalised once, on the way in, rather than at every comparison?
- Do both rankings use the full tie-break chain, and does a profile update show up in the very next query?
- Are `apply` (check eligibility, check duplicate, append) and `closeJob` atomic with respect to each other, so no application slips in after a close?
- Does a rejected call leave all state unchanged, and does `topCandidates` avoid a full scan with a full sort where it can?

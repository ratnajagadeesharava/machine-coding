# Rubric — 112 Parking Lot with Allocation Strategies (Java)

| Area | Weight |
|---|---:|
| Functional correctness | 40% |
| Design quality (model, API, responsibilities) | 25% |
| Code quality | 10% |
| Testing | 10% |
| Concurrency (declared scope, no shared mutable surprises) | 10% |
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

- Is the global spot numbering implemented exactly as specified (floor blocks, two-wheelers before four-wheelers inside a floor)?
- Is the allocation rule a replaceable decision (selectable and switchable at runtime) rather than an `if` scattered through `park`, and was that justified by the two required strategies?
- Does `MOST_FREE_FLOOR` count only compatible free spots, and are all ties resolved exactly as specified?
- Do `park` and `unpark` keep the free structures, the ticket table and the registration index consistent, including on failure paths?
- Is a freed spot immediately allocatable, and can a stale ticket never free somebody else's spot?
- Is the duplicate-registration check atomic with the allocation?

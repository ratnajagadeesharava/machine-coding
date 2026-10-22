# Rubric — 107 Excel-Like Autofill Engine (Java)

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
- Are collections, locks and executors chosen on purpose (not `synchronized` everywhere)?
- Is mutable state encapsulated; are value types immutable (records / final fields)?

## Problem-specific

- Is each series kind recognised and continued in one place, so a new kind can be added without touching the others?
- Is decimal arithmetic exact, and is the output formatting (no exponent, no trailing zeros, no `-0`) right on every path?
- Are letter labels treated as bijective base 26 (`Z` then `AA`, `ZZ` then `AAA`), not as ordinary base 26?
- Are the unrecognised-input rules (mixed kind, mixed case, non-arithmetic, malformed, empty) decided, stated and tested, and is validation done before looking at `count`?
- Is the decreasing-letters stop rule implemented as specified (shorter result, not an error and not a wrap)?
- Is every consecutive pair of seeds checked, and is the single-seed case handled?

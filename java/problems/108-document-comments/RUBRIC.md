# Rubric — 108 Document Comment Service (Java)

| Area | Weight |
|---|---:|
| Functional correctness | 40% |
| Design quality (model, API, responsibilities) | 15% |
| Code quality | 10% |
| Testing | 10% |
| Concurrency (declared scope, no shared mutable surprises) | 20% |
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

- Are the ids a single service-wide sequence, assigned only to accepted comments, and assigned and appended together so list order equals id order under concurrency?
- Is validation done before any state is touched (blank beats unknown), so a rejected call leaves nothing behind?
- Does `comments` return a real copy, and can a reader never observe a gap or a half-added comment?
- Is the set of documents fixed and copied at creation, with per-document state that does not need a global lock?
- Is the output format exactly `reviewerId: text`, with the text untouched?

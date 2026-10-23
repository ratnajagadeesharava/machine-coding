# Rubric — 117 Search Query Pipeline (Java)

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

- Can a new step be added without editing any existing step or the code that runs them? Is the built-in set expressed the same way as a registered step (no special cases in the runner)?
- Is the failure policy in one place (not a try/catch inside every step), and does a failed step leave the query as it was?
- Is the spell rule exact (edit distance 1, exactly one candidate), and are the synonym group and filter collapse formats exact?
- Does a `process` call run against one consistent snapshot of the step list while registrations happen concurrently?
- Is the trace precise: only steps that changed something, `:failed` entries, nothing for disabled or removed steps?
- Are the query text format (OR groups) and the term model kept separate enough that rank and filter do not each re-invent a parser?

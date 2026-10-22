# Rubric — 115 Autocomplete / Typeahead Service (Java)

| Area | Weight |
|---|---:|
| Functional correctness | 40% |
| Design quality (model, API, responsibilities) | 20% |
| Code quality | 10% |
| Testing | 5% |
| Concurrency (declared scope, no shared mutable surprises) | 10% |
| Performance and memory | 15% |

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

- Is a suggestion served without scanning every term (cost related to prefix length and k, not to the number of terms), and can the candidate state the cost of `record`?
- Is the top-k answer still correct after a low-ranked term overtakes the leaders, and after k exceeds the matches?
- Are ties always alphabetical, for both ranking rules and for the personalised query?
- Is normalisation applied identically to terms and prefixes, and are rejected calls free of side effects?
- Do concurrent writers keep exact totals, and can readers run without waiting for a global lock?
- Is the ranking rule replaceable without editing the others, and is personalisation a layer on top of the global data rather than a copy of it?
- Can the candidate estimate the memory per term and name two ways to shrink it?

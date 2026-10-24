# Rubric — 131 Shortest Distance and Nearest Landmarks (Rust)

| Area | Weight |
|---|---:|
| Functional correctness | 45% |
| Design quality (model, API, responsibilities) | 20% |
| Code quality | 10% |
| Testing | 10% |
| Concurrency (declared scope, no shared mutable surprises) | 5% |
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
- Are collections and locks chosen on purpose (not one big `Mutex` held for a whole query)?
- Is mutable state encapsulated; are the error cases modelled as types rather than strings?

## Problem-specific

- Is every distance an `i64` all the way through (storage, sums, the "unreachable" marker), with no 32-bit truncation or overflow?
- Are the shortest-route and the tie-break rules exactly as specified (equal total, then smallest name sequence), including for parallel roads?
- Do nearest-landmark results use the exact ordering (distance, then name), count the source at 0, skip unreachable ones, and normalise types?
- Does any query answer from data that an earlier update invalidated? Is there any cache at all, and if so why is it safe?
- Does a query on the 150 x 150 grid cost roughly the size of the part of the map it needs, not the whole map squared or a per-query rebuild of the graph?
- Are invalid inputs rejected before any state is touched, and does a query see one consistent map during concurrent updates?

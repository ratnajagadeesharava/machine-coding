# Rubric — 128 Local Search with Ranking (Rust)

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
- Are traits/types justified, or did you reach for a trait by reflex?
- Are collections, locks and smart pointers chosen on purpose (not `Mutex` around everything)?
- Is mutable state encapsulated; are value types immutable (no needless `pub` fields, no needless clones)?

## Problem-specific

- Is the score computed exactly as specified (including the stacked equality bonus) in `i64` arithmetic, with the three-level tie-break?
- Are text matching, case handling and the category-equals-query rule deliberate and consistent between filtering and bonuses?
- Does the search keep only `k` candidates at a time instead of sorting everything, and survive `k = i32::MAX`?
- Is an update atomic for readers, and does a rejected add/update leave state untouched?
- Is the cost of a search stated honestly (full scan vs. something smarter), and does the candidate know when the simple version stops being enough?
- Is the ranking logic separate from the storage/filtering so a boost or sponsored slot could be added without touching the store?

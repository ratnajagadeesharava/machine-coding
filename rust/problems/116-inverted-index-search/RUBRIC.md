# Rubric — 116 In-Memory Search Engine (Rust)

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
- Is mutable state encapsulated; are value types immutable (plain structs, no needless clones)?

## Problem-specific

- Is idf computed from the live collection at query time (after deletes and updates), with `N` including token-less documents?
- Is the tie rule exact (score then doc id), including for mathematically equal scores that arrive from different tf fractions?
- Does a search look only at documents that contain a query term (postings), rather than scanning every document?
- Are delete and update symmetrical with add: no stale postings, no empty entries left behind, no half-applied update visible?
- Do duplicate query terms count once, and does an unknown term behave correctly in both modes?
- Is pagination consistent with the full ranking, and is the sort not done on more than needed?
- Are the tokeniser and the scoring replaceable without touching the index code (asked as a follow-up, but is the seam natural)?

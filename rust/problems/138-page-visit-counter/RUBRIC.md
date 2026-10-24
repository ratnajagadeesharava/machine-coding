# Rubric — 138 Multi-Threaded Page Visit Counter (Rust)

| Area | Weight |
|---|---:|
| Functional correctness | 35% |
| Design quality (model, API, responsibilities) | 15% |
| Code quality | 10% |
| Testing | 5% |
| Concurrency (no lost updates, consistent snapshot, no deadlock) | 25% |
| Performance and memory | 10% |

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
- Are collections, locks and counters chosen on purpose (not one big lock or `synchronized` everywhere)?
- Is mutable state encapsulated; are value types immutable?

## Problem-specific

- Is normalisation exactly as specified (order of steps, root path, repeated slashes, no scheme, case rules) and applied by every operation, including reads and `reset`?
- Are totals, uniques and per-visitor counts exact under 16 writers (no lost update, no check-then-act race when a page or visitor is first seen)?
- Does memory depend on distinct (page, visitor) pairs only?
- Can the candidate say precisely what a snapshot guarantees, and does the implementation deliver it (not just a per-page copy)? What do writers pay?
- Are `reset` and `resetAll` safe against concurrent visits (a visit lands wholly before or wholly after, never half)?
- Is the ranking tie-break exactly as specified, and is `topPages` affordable on a big site (not a full sort of everything for k = 1)?
- Is the failure path clean: an invalid call leaves no trace (no empty page entry created by a rejected visit)?

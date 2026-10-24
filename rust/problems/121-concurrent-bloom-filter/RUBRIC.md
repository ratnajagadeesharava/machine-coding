# Rubric — 121 Concurrent Bloom Filter

| Area | Weight |
|---|---:|
| Functional correctness | 30% |
| Design quality (model, API, responsibilities) | 15% |
| Code quality | 10% |
| Testing | 5% |
| Concurrency (declared scope, no shared mutable surprises) | 30% |
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
- Is the sizing computed in one place and exposed through the accessors? Is integer/float conversion done deliberately (no `usize`/`u32` overflow of `m`, no truncation instead of `ceil`)?
- Is the bit array's memory visibility and atomicity argued, not assumed (a `long[]` with plain `|=` is not enough)?

## Problem-specific

- Are `m` and `k` exactly the documented values (ceil, round half up, at least 1)?
- Is the item-to-positions mapping deterministic (no process-random seeds, no identity hashes) and independent of insertion order?
- Does `add` report "changed" from the bits it actually flipped, and can two threads both claim the same flip?
- Is a probe correct while a concurrent add is half way through its `k` positions (never a false negative for an item whose `add` has returned)?
- Is the estimate computed from a consistent reading, with the saturated and empty cases handled?
- Is validation done before any bit is touched (including in `addAll`)?

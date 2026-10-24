# Rubric — 140 Custom HashMap (Rust)

| Area | Weight |
|---|---:|
| Functional correctness | 40% |
| Design quality (model, API, responsibilities) | 15% |
| Code quality | 10% |
| Testing | 10% |
| Concurrency (declared scope, atomic operations) | 10% |
| Performance (average and worst case) | 15% |

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
- Are locks chosen on purpose and held for as short as possible? Is `unsafe` avoided or justified?
- Is mutable state encapsulated; are ownership and borrowing used rather than fought (no needless cloning of whole tables)?

## Problem-specific

- Did the candidate avoid built-in maps and sets for the storage, and say what they used instead?
- Is the bucket index always in range for any `u64` hash, including `u64::MAX` (no overflow when mixing or adding)?
- Is the hasher called exactly once per keyed operation, including when the table grows?
- Is the growth rule implemented exactly (check after a new key, doubling once, no shrinking, no overflow in `size * 100`)?
- Is insertion order kept stable across resizes, re-puts and removals, and is `remove` independent of the map size?
- Can the candidate state the average and worst-case cost of each operation, and what an all-colliding hasher does to them?
- Are compound operations (`putIfAbsent`, resize, `keys()`) atomic with respect to other threads?

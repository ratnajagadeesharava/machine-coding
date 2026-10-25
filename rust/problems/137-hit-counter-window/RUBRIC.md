# Rubric — 137 Configurable Time Window Hit Counter

| Area | Weight |
|---|---:|
| Functional correctness | 30% |
| Design quality (model, API, responsibilities) | 15% |
| Code quality | 10% |
| Testing | 5% |
| Concurrency (declared scope, no shared mutable surprises) | 15% |
| Performance and memory | 25% |

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
- Java: are `long` counts used consistently (no `int` overflow), is the shared clamp value updated atomically, are returned lists defensive copies?
- Rust: is the last-seen time inside the same critical section as the data, and are `u64` subtractions safe when `now < window`?

## Problem-specific

- Is the window boundary exact (`age == window` expired) for both `count` and `countInLast`, and is it still right with a window of 1 ms and at the start of time (`now < window`)?
- Is memory proportional to the number of distinct timestamps/buckets in the window, not to the number of hits?
- Who removes expired data? Does memory stay bounded when a key is written but never read, and when a key goes quiet forever?
- Is the monotonic clamp applied to reads as well as writes, and kept consistent across threads?
- Does `setWindow` evict with the old window before applying the new one (no resurrection), and is it safe next to concurrent hits?
- Is `topKeys` ordered by count then key, free of zero-count keys, and what does it cost with a million keys?
- Is `countInLast` efficient, or does it scan every stored record?
- Is validation done before any mutation?

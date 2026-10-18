# Rubric — 23 Notification System (Rust)

| Area | Weight |
|---|---:|
| Functional correctness | 35% |
| Design quality (model, API, responsibilities) | 35% |
| Code quality | 15% |
| Testing | 15% |

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
- Is ownership clear — who owns which state, and where are borrows vs clones used?
- Are traits introduced only where polymorphism is needed (enum vs trait object vs generics)?
- Is shared state (`Arc`/`Mutex`/`RwLock`/atomics/channels) used only where actually required?
- Does the code pass `cargo fmt --check` and `cargo clippy` cleanly?

## Problem-specific

- Is the send flow clear: validate, render once, filter eligible channels, attempt with retry, report?
- Are the preference defaults and the order of checks implemented exactly (zero gateway calls for refused requests)?
- Is the gateway used as an injected dependency rather than reached around?
- Is rendering single-pass, and is the retry rule expressed in one place?
- Is the design free of speculative abstractions, yet obviously extendable for a new channel?

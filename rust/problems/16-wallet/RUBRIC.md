# Rubric — 16 Digital Wallet (Rust)

| Area | Weight |
|---|---:|
| Functional correctness | 30% |
| Design quality (model, API, responsibilities) | 20% |
| Code quality | 10% |
| Testing | 10% |
| Concurrency | 25% |
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
- Is ownership clear — who owns which state, and where are borrows vs clones used?
- Are traits introduced only where polymorphism is needed (enum vs trait object vs generics)?
- Is shared state (`Arc`/`Mutex`/`RwLock`/atomics/channels) used only where actually required?
- Does the code pass `cargo fmt --check` and `cargo clippy` cleanly?

## Problem-specific

- Is the transfer atomic and deadlock-free (lock ordering or equivalent)?
- Is the idempotency check atomic with applying the operation and with recording the key, and is a refused request not burning its key?
- Are results and validation order exactly as specified, with no partial effects on refusal?
- Is the model small (balances + statements + key set) rather than a hierarchy of services?

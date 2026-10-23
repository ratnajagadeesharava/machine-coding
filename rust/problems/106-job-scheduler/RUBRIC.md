# Rubric — 106 Job Scheduler (Rust)

| Area | Weight |
|---|---:|
| Functional correctness | 45% |
| Design quality (model, API, responsibilities) | 20% |
| Code quality | 10% |
| Testing | 10% |
| Concurrency (declared scope, no shared mutable surprises) | 5% |
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

- Is eligibility ("has ALL required capabilities") computed correctly, and are selection and the counter update one atomic step?
- Is the selection criterion a first-class, replaceable concept, so a third criterion is a small addition rather than an edit of the scheduling flow? Is the abstraction justified by the requirement, not by reflex?
- Is the tie-break (smallest machine id, plain string order) deterministic and applied under every criterion?
- Does a failed call (no eligible machine, duplicate job, unknown criterion, double finish) leave no trace?
- Can two threads never assign on stale counters, and can a job be finished by exactly one caller? Is the locking granularity reasoned about (one lock vs. per-machine)?
- What does one submission cost with 100,000 machines, and is there a cheaper index (by capability) worth discussing?

# Rubric — 135 Container Orchestrator (Rust)

| Area | Weight |
|---|---:|
| Functional correctness | 40% |
| Design quality (model, API, responsibilities) | 20% |
| Code quality | 10% |
| Testing | 5% |
| Concurrency (declared scope, no shared mutable surprises) | 15% |
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
- Is ownership clear — who owns which state, and where are borrows vs clones used?
- Are traits introduced only where polymorphism is needed (enum vs trait object vs generics)?
- Is shared state (`Arc`/`Mutex`/`RwLock`/atomics/channels) used only where actually required?
- Does the code pass `cargo fmt --check` and `cargo clippy` cleanly?

## Problem-specific

- Is the fit test exactly `free >= request` on both resources, with leftover = remaining CPU + remaining memory for BEST/WORST_FIT, and ties by node id?
- Is the retry pass a single walk in (priority desc, submission asc) order that skips containers that do not fit, without head-of-line blocking?
- Does a container keep its original submission order when it is evicted or displaced?
- Is the preemption victim selection exact (strictly lower priority, lowest priority then newest, minimal prefix, fewest evictions, node-id tie, all-or-nothing)?
- Does `removeNode` report the state after rescheduling, in container-id order, and never preempt?
- Is each public operation atomic with its own retry pass, and can no thread observe an over-committed node?
- Are input errors separated from normal results and do they leave state unchanged?

# Rubric — 114 Elevator Request Feasibility (Rust)

| Area | Weight |
|---|---:|
| Functional correctness | 45% |
| Design quality (model, API, responsibilities) | 20% |
| Code quality | 10% |
| Testing | 10% |
| Concurrency (declared scope, no shared mutable surprises) | 10% |
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
- Are traits/enums justified, or did you reach for a trait by reflex?
- Is the lock scope deliberate (one `Mutex` around the whole decision), and is a failed request free of partial writes?
- Are value types plain structs, and is the simulation working on a copy rather than cloning half the world by habit?

## Problem-specific

- Does feasibility come from simulating the future of the circuit with the newcomer added, rather than from per-request counters that drift out of sync?
- Are all three checks present: capacity at every stop, the newcomer's own limit, and the limit of every earlier rider?
- Are riders who are already onboard credited with the stops they have already sat through?
- Does the capacity check release riders before boarding at the same stop, and take direction into account for who boards?
- Is the decision made on a copy (or an undoable change) so a rejection leaves nothing behind?
- Does `step()` do exactly what `plan()` predicted?
- Is check-then-record atomic under concurrent requests?

# Rubric — 136 Fantasy Team Leaderboard (Rust)

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

- Is a player's match points stored once and each team's score derived from it (so a team created later is right), and is the x2 / x1.5 weighting exact (half points, no floats)?
- Does one event update every team containing the player, and can a reader ever see it applied to only some of them?
- Is the leaderboard ordering (score desc, creation order) and competition ranking exactly as specified, including a tied group cut by `k`?
- What does `leaderboard(10)` cost with 50,000 teams, and what does one event cost for a player in thousands of teams? Did the candidate reason about both?
- Do failed operations (bad roster, over-correction, bad event) leave every structure untouched, including the team id and the correction counters?
- Is locking granularity argued (one lock, striped, snapshot) and is the snapshot guarantee actually delivered?

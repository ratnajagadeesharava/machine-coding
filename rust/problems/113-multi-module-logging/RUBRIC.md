# Rubric — 113 Multi-Module Logging Framework (Rust)

| Area | Weight |
|---|---:|
| Functional correctness | 35% |
| Design quality (model, API, responsibilities) | 20% |
| Code quality | 10% |
| Testing | 10% |
| Concurrency (shared buffer, ordering, shutdown) | 20% |
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
- Are traits justified, or did you reach for `dyn` by reflex?
- Are channels, locks and threads chosen on purpose (not `Mutex` around everything)?
- Is mutable state encapsulated; is ownership clear and are `Arc` clones deliberate?

## Problem-specific

- Is the sequence number assigned in exactly one place, by one owner, and is the failure rule (no number consumed) stated and enforced?
- Is the acknowledgement really after the sink write, and is the log visible to `query` by the time the caller sees its number?
- Is the buffer genuinely bounded with back pressure (no unbounded queue, no dropping), and is shutdown deterministic (no leaked thread, no waiter left hanging)?
- Does a throwing or failing sink leave the consumer alive and the other destination working?
- Are filters validated before use, with the level treated as a threshold and the range as inclusive?
- Does `query` return a stable snapshot without blocking producers for long?

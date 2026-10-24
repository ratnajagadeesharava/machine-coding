# Rubric — 126 Alert Monitoring System (Java)

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
- Are collaborators injected (the clock, the notifier) rather than reached statically?
- Are the string names (comparators, aggregations, severities) converted to a type once at the boundary instead of compared as strings throughout?
- Is mutable state encapsulated (no leaked internal lists from `history`/`activeAlerts`)?

## Problem-specific

- Is the rule life cycle (`OK`, `PENDING`, `FIRING`, `RESOLVED`) modelled explicitly, with each transition and its side effects (history entry, notification, pending timestamp) in one place, rather than scattered flags and `if` chains over several fields?
- Are the boundaries exact: half-open window `(now - windowMs, now]`, `forMs` reached at equality, cooldown boundary, silence ending at `untilMs`, strict versus inclusive comparators?
- Is the empty-window rule right (condition false, value `0.0`, `COUNT` compared as 0)?
- Is the "notify" side separated from the "decide" side so channels, severity routing and failing notifiers are handled in one place and one failing channel cannot break the rest?
- Is the aggregate a replaceable piece of behaviour, so a sixth aggregation or a rate-of-change rule is a small addition rather than an edit of the evaluation flow? Is that abstraction justified by the requirement?
- Does silence leave state tracking untouched, and does it avoid starting the cooldown?
- Is retention correct per metric (largest window among that metric's rules) and does it never remove samples a rule still needs?
- Is `record` safe against concurrent `evaluate`, with no lost samples and no half-seen state? Is the notifier called under a lock, and if so is that a deliberate decision?
- What does an evaluation cost with many rules and many samples, and what would you change (per-metric sorted samples with binary search, running aggregates, a monotonic deque for min/max)?

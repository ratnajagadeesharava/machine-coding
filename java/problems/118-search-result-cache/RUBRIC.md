# Rubric — 118 Search Result Cache

| Area | Weight |
|---|---:|
| Functional correctness | 35% |
| Design quality (model, API, responsibilities) | 15% |
| Code quality | 10% |
| Testing | 5% |
| Concurrency (declared scope, no shared mutable surprises) | 25% |
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
- Are immutable values used for entries and results (`List.copyOf`, records)? Is the exception contract (original type, not a wrapper) preserved for waiters?
- Is the lock scope small (no I/O under a lock)? Are `ConcurrentHashMap.compute`-style calls not used to run the backend under a bucket lock?

## Problem-specific

- Is the cache key exactly the normalised query, and is the backend given that same text?
- Is every entry checked for both age (boundary: expired at exactly ttl) and index version, and is the version read before the backend call?
- Is recency updated on hits and on stores, with the capacity bound never exceeded?
- Is the backend called with no lock held, and does a slow call for one query leave others unaffected?
- Is there exactly one backend call for N identical concurrent queries, with the result (or failure) delivered to all waiters and failures never stored?
- What happens to an in-flight call when `invalidate`/`invalidateAll` runs? Is the stale result kept out of the cache?
- Are the statistics exact under contention (every valid get is one hit or one miss) and is `size()` honest about dead entries?
- Are returned lists defended against the caller and against later mutation by the backend?

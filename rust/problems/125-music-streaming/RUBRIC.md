# Rubric — 125 Music Streaming and Search

| Area | Weight |
|---|---:|
| Functional correctness | 40% |
| Design quality (model, API, responsibilities) | 20% |
| Code quality | 10% |
| Testing | 5% |
| Concurrency (declared scope, no shared mutable surprises) | 15% |
| Performance (search, rankings) | 10% |

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
- Are classes/interfaces justified, or did you reach for an interface by reflex?
- Are collections, locks and executors chosen on purpose (not a lock around everything without a reason)?
- Is mutable state encapsulated; are value types immutable?

## Problem-specific

- Is the match rule exactly "every token is a prefix of some word of title or artist", case-insensitive, with whitespace-only tokenisation?
- Is the ranking exactly plays desc, lower-cased title asc, id asc, and is the same ordering reused for search and top songs rather than copied?
- Does the history hold 10 distinct songs with move-to-front, and does `play` update global count, user count and history as one step?
- Are recommendations computed from per-user, per-artist play counts, excluding anything the user has played, with the three-key ordering?
- Do `playNext`, `play` and counters hold up under contention: nothing lost, nothing popped twice, no deadlock between user state and song state?
- Does search avoid scanning every song for every query?
- Are errors separated from normal results and do they leave state unchanged (including a rejected duplicate playlist entry)?

# Rubric — 123 · Snake Game With Food And Score (Java)

| Area | Weight |
|---|---:|
| Functional correctness | 50% |
| Design quality (model, API, responsibilities) | 20% |
| Code quality | 10% |
| Testing | 10% |
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
- Is concurrency safe (or explicitly and correctly declared out of scope)?
- Are performance characteristics reasonable for the stated constraints?

**Language-specific**
- Are classes/interfaces justified, or did you reach for an interface by reflex?
- Are collections chosen on purpose (deque + hash set rather than scanning a list)?
- Is mutable state encapsulated; are value types immutable (records / final fields), and is the input list copied?

## Problem-specific

- Is the order of checks right: invalid input, already over, reversal, wall, body (with the leaving-tail exception), then eat or move?
- Is the tail released only when nothing is eaten, and is the collision check done against the right set of cells?
- Is food spawning exactly as specified: skip entries under the snake after the move was applied, skipped entries gone for good?
- Does a fatal move leave head, body, score and food untouched, and does the game then stay frozen?
- Are the win cases covered (full board on eating, the 1x1 board at creation) and kept apart from a loss?
- Is a move O(1) (hash set or grid for occupancy, a deque for the body) and creation independent of the board area?

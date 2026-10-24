# Rubric — 130 Ludo (Java)

| Area | Weight |
|---|---:|
| Functional correctness | 45% |
| Design quality (model, API, responsibilities) | 20% |
| Code quality | 10% |
| Testing | 15% |
| Performance | 5% |
| Communication of assumptions | 5% |

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

**Language-specific**
- Are classes/interfaces justified, or did you reach for an interface by reflex?
- Is mutable state encapsulated; are value types immutable (records / final fields)?

## Problem-specific

- Is the turn modelled so that "roll pending" and "move pending" are explicit, and every invalid call is rejected without changing anything?
- Is the conversion from (colour, progress) to a track cell done once, and correct for all four colours including the wrap past cell 51?
- Is the exact-count finish right (progress 55 + 3 is not allowed, + 2 is), and does a yard token need exactly a 6?
- Are the extra-turn rules (6, capture, HOME) and the third-six forfeiture applied in one place, with the six counter reset in the right situations?
- Is capture evaluated per opponent colour with the block rule and the safe cells, only on shared cells?
- Does the candidate's own test suite include a short hand-checked scenario (not just "it runs")?
- Is the next-player logic free of special cases for the number of players?

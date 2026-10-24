# Rubric — 133 Spreadsheet Grid (Java)

| Area | Weight |
|---|---:|
| Functional correctness | 40% |
| Design quality (model, API, responsibilities) | 20% |
| Code quality | 10% |
| Testing | 10% |
| Concurrency (declared scope, no shared mutable surprises) | 5% |
| Performance and memory (sparse data, structure operations) | 15% |

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
- Are collections and locks chosen on purpose (not `synchronized` everywhere without a reason)?
- Is mutable state encapsulated; are value types immutable (records / final fields)?

## Problem-specific

- Do memory and time follow the used cells (a million-by-702 grid is never allocated or walked), including `clearRange` over everything and `usedRange`/`rowCount`/`columnCount`?
- Are styles immutable values that equal combinations share, so that a million styled cells do not cost a million style objects, and is `distinctStyleCount` correct after styles are changed back and cells are cleared?
- Is the style normalisation (case, defaults) and the "styled but empty" cell handled consistently, so that a cell equal to the default disappears from the used range?
- Number display: shortest-decimal based, half away from zero, no `-0`, no exponent; are format and value kept separate so a format change re-renders?
- Is `copy` correct for every overlap direction, with empties overwriting, and do errors leave the grid untouched (validate first, then mutate)?
- Do row/column insertion and deletion cost something proportional to what moves (or less), and do cells pushed off the grid vanish?
- Is the sort stable, with the group order and the descending rule exactly as specified?
- Are compound operations atomic for other threads (no reader sees half of a `move` or a structure change)?

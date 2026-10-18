# Interviewer notes — 04 Coffee Machine

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- The alphabetically-first missing ingredient is reported.
- A recipe that mentions an ingredient that was never refilled is refused with that ingredient.
- `lowStock` is strictly-less-than; fully consumed ingredients appear for threshold ≥ 1.
- Replacing a recipe changes what the next dispense consumes.

## Likely interviewer questions
- Where does the all-or-nothing check-and-consume happen? What if two outlets check at once?
- Single lock vs per-ingredient locks: what is the deadlock risk, and how do you order acquisitions?
- Is `Ingredient` a class, an enum, or a string key? What would change if units/expiry were needed?
- How would "alert once when below threshold" be built without polling?

## Invariants
- `stock(i) ≥ 0` always; `initial + refills − Σ consumed(OK dispenses) = stock`.
- A refused dispense changes no stock.
- A recipe is observed either entirely old or entirely new by a concurrent dispense.

## Common mistakes
- Check-then-consume without atomicity (two outlets both pass the check).
- Consuming ingredient-by-ingredient and "rolling back" on failure (visible intermediate states, negative stock).
- Per-ingredient locks acquired in arbitrary order (deadlock).
- Reporting the first missing ingredient in map-iteration order (non-deterministic).

## Over-engineering traps
- `Beverage` class hierarchy with `Latte`, `Espresso` subclasses when they differ only by data.
- Decorators for milk/sugar in V1.
- `IngredientRepository` interface + implementation.

## Concurrency discussion
- Single `ReentrantLock` around check+consume is correct; per-ingredient striping needs ordered locking. Compare throughput.
- Optimistic approach with compare-and-swap on a snapshot of stock.

## Trade-offs worth probing
- Coarse correctness-first lock vs fine-grained complexity: what is the measured benefit at 16 outlets?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.

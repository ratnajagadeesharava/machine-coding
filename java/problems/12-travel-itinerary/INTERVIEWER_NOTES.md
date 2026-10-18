# Interviewer notes — 12 Travel Itinerary Planner

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- Removing the middle activity can create a violation between its former neighbours (non-metric travel times) → removal refused.
- Touching intervals (end == start) at the same location are fine; at different locations only when travel time is 0.
- Adding an activity *between* two others must check both neighbours.
- Budget exactly equal to the total is allowed; one more unit is refused.
- Travel time set after planning does not invalidate existing plans.

## Likely interviewer questions
- What are your entities and which are just values (a time range? a money amount?)? Who owns the per-day ordering?
- Which structure keeps a day's activities sorted and makes neighbour lookups cheap? Complexity of add/remove/agenda?
- How do you express "all-or-nothing" checks — validate before mutate, or mutate and roll back?
- Where do travel times live? A matrix, a map keyed by an unordered pair? How do you key an unordered pair?
- How would multi-city travel (V2) change the model — is the travel time still a number?

## Invariants
- Within a day: activities are non-overlapping and ordered; consecutive different-location pairs respect travel time.
- `Σ cost ≤ budget`; activity ids unique per trip.
- Every refused mutation leaves state untouched.

## Common mistakes
- Checking only the previous neighbour (or only overlap, ignoring travel time).
- Treating `[a,b]` as closed instead of half-open (refusing touching intervals).
- Unordered pair keyed as `(a,b)` only, so `(b,a)` is not found.
- Removal that ignores the new adjacency.
- Mutating before validating budget and leaving a partial change on refusal.

## Over-engineering traps
- `Itinerary`, `DayPlan`, `Activity`, `Location`, `Budget`, `TravelTimeProvider` interface, `ConstraintChecker` chain, `Planner` facade... A couple of well-chosen types plus a sorted map is plenty.
- A validation framework with rule objects when four rules fit in one method.

## Scalability discussion
- Per-day balanced tree or sorted vector with binary search; per-trip locks for concurrent edits.

## Trade-offs worth probing
- Rule objects (extensible) vs a single validation function (clear). V2/V3 might tip the balance.

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.

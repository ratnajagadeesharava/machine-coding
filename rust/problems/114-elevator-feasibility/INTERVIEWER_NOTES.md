# Interviewer notes — 114 Elevator Request Feasibility

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## The reference model (one valid reading of the README)
The lift sits at a position on a ring of `2*(floors-1)` positions: floor 0 heading up, floors 1..F-2 heading up, floor F-1 heading down, floors F-2..1 heading down, then back to floor 0. `request` copies the rider list, adds the newcomer, and simulates stops from the current ring position until nobody is waiting or onboard; the copy is thrown away if any check fails. A rider's stop counter is reset on boarding and incremented at every later stop where they stay inside; it is checked when they leave. Candidates may implement it differently as long as the observable answers match.

## Hidden edge cases (see `interviewer/`)
- A stop at the floor where the lift is standing counts for riders already inside (doors reopen): `limit 0`, one rider onboard, a second rider at the same floor is rejected.
- A rider that has already sat through some stops: the counter must be carried across `step()` calls, not recomputed from the plan.
- 100 rejections in a row must leave plan, position, load and statuses byte-for-byte identical (a simulation that mutates the live state shows up here).
- Capacity freed by deliveries: the lift accepts again after the riders leave.
- A full lap: an upward rider requested while the lift is heading down waits for the turn; a downward rider above waits for the next turn at the top.
- A big group that exactly fits (4 into 4) boards after another group of 4 leaves at the same stop.
- Concurrent requests racing with `step()`: the load never exceeds capacity at any observation, and every accepted rider is delivered at the end.
- A 500-floor building with 300 requests runs in well under a minute (the naive full re-simulation per request is intended to pass; V6 asks for better).

## Likely interviewer questions
- Walk me through `r1 = 0->9`, then `r2 = 3->5` with limit 1: exactly what do you simulate, and which counter breaks first?
- Why is a rejection "free" in your design? What would a partially applied state look like?
- You count stops already made: where do you store that, and who resets it?
- Does a request at the current floor reopen the doors? Defend your reading and show where the README says so.
- What changes if capacity were measured before riders get off at a stop (a different but plausible reading)?

## Invariants
- At every stop of every plan: load after boarding `<=` capacity.
- Every promised rider's intermediate stops `<=` the limit, in the plan that results from any accepted request.
- `plan()` is a pure function of the current state; `step()` makes plan()[0] happen and nothing else.
- A rejected request leaves no trace; an accepted one is recorded atomically with the check.

## Common mistakes
- Checking only the newcomer's own stop count, so earlier riders' promises are silently broken.
- Counting stops globally ("the plan has 5 stops, limit is 3") instead of per rider between boarding and exit.
- Forgetting stops already made for riders who are onboard (the counter restarts when the plan is recomputed).
- Counting the boarding or the exit stop as intermediate (off-by-one both ways).
- Checking capacity before riders have got off at the same stop, rejecting a legitimate swap.
- Boarding every waiting rider at a floor regardless of direction, or ignoring that floor 0 / the top floor have only one heading.
- Mutating live state while simulating, then forgetting to undo it on rejection.
- Treating `persons > capacity` as an exception rather than a rejection.
- Check-then-act without a lock: two concurrent requests are each feasible alone but not together.
- A `plan()` that disagrees with what `step()` then does (two different movement implementations).

## Over-engineering traps
- A strategy hierarchy of scheduling algorithms (SCAN, LOOK, nearest car) when exactly one circuit is specified.
- An event bus of door/arrival events, a scheduler thread, or real timers for movement.
- Modelling cabin, doors, buttons and display classes that the contract never touches.

## Design and scaling discussion
- Feasibility by simulation is O(stops x riders) per request; with a sorted structure per stop it drops to O(stops). The answer must stay identical.
- Incremental evaluation: adding a rider only inserts at most two stops into the plan and only changes loads and counters for riders whose rides contain them.
- With multiple lifts, rejections become assignments: the choice is greedy, so ask what regret looks like.
- A real controller re-optimises the plan, which breaks promises unless the promise check is part of re-optimisation.

## Trade-off questions to ask when the candidate is done
- Fixed circuit vs. reversing at the farthest request: what does each do to the answers and to the comfort limit?
- Why accept based on future simulation instead of a conservative bound (e.g. reject anything that might add a stop)? What does each cost the building?
- How would you show a client *why* the request was rejected (which rider, which stop)?
- What is the blast radius of a bug in the check: an overpromised ride vs. an unnecessary rejection?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.

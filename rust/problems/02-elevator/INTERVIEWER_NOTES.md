# Interviewer notes — 02 Elevator System

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- A cabin request for the current floor opens doors without moving.
- Reversal: destination 10, then after arriving destination 2 → served in order `[10, 2]`.
- No elevator moves when nothing is requested.
- Large random workloads: every request served.

## Likely interviewer questions
- How do you represent "stops"? Per-elevator sorted sets? Two sets (up/down)? A request queue with state?
- Who decides which elevator takes a hall call — the elevator, the controller, a policy? What information does it need?
- What happens to a hall call already assigned if a better elevator becomes available?
- How do you avoid starvation at far floors under constant load?
- How would you test dispatch quality, not just correctness?

## Invariants
- Elevator position stays in `[0, F-1]`; it moves at most one floor per step.
- An elevator never both moves and opens doors in the same step *unless* it arrives and opens (arrival opens in the same step).
- A hall call is owned by exactly one elevator until served.
- Every accepted request is served in finite steps.

## Common mistakes
- Serving a hall call from *every* elevator, or re-assigning each step so nothing converges.
- Direction state that is not updated when stops run out (elevator oscillates or sticks).
- Treating `UP` calls as servable when travelling `DOWN` (violating the contract).
- Forgetting stops added while moving.
- Unbounded `O(F)` scans each step.

## Over-engineering traps
- State pattern with `MovingUpState`, `MovingDownState`, `IdleState`, `DoorOpenState` classes before the logic needs it; an enum + a function may be enough.
- A `Dispatcher` interface with a single implementation in V1 (it earns its place in V3).
- `Request` class hierarchy for hall vs cabin.

## Scalability / concurrency discussion
- Moving to real time and threads: elevators as actors with a mailbox vs a central scheduler with locks.
- Dispatch is an optimization problem: cost function (distance, direction, load, pending stops). Complexity of re-evaluating on every call.

## Trade-offs worth probing
- Per-car stop sets vs central pending-request pool.
- Eager assignment at call time vs late binding at arrival.

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.

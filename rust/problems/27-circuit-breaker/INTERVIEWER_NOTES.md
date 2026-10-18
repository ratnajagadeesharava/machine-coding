# Interviewer notes — 27 Circuit Breaker

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- Threshold of 1 and probe requirement of 1.
- Rejected calls during the cool-down must not extend it.
- A trial failure after some trial successes discards the successes.
- The failure count restarts from zero after the guard has closed again.
- State is time-derived: no call is needed to observe the transition to probing.

## Likely interviewer questions
- What are the states and which transitions exist? Is the cool-down check done by a timer thread or lazily on access? Why?
- Where do you hold the lock and where not? What happens if the operation blocks for 30 seconds?
- How do you make sure only one trial is admitted when 100 threads arrive at the same instant?
- A call admitted while closed finishes after the guard tripped. Does its result count?
- What if the operation throws? What if it never returns?

## Invariants
- At most one trial is in flight at any time.
- A rejected call never runs the operation.
- The failure count never reaches the threshold without the guard being tripped.
- Exactly the `N`-th consecutive failure trips the guard (no off-by-one).

## Common mistakes
- Holding a mutex around the operation (serialises everything and the "one trial" test passes for the wrong reason, but state queries block).
- Checking `state == HALF_OPEN` and then setting "trial in flight" in two separate critical sections (two trials get admitted).
- Forgetting to release the trial slot when the operation throws.
- `>` instead of `>=` for the cool-down boundary; restarting the cool-down on rejected calls.
- Counting failures that are not consecutive; not resetting the count on success or on closing.

## Over-engineering traps
- A `State` interface with three state classes, a context object, transition tables, listeners, a metrics subsystem in V1. Three labels plus a handful of fields guarded by one lock may be all you need.
- A background thread to flip OPEN to HALF_OPEN.

## Concurrency discussion
- One short critical section to decide admission and to record the outcome; the operation runs outside of it.
- Lock-free variant: a single atomic holding (state, counters, timestamps) is possible but harder to get right than the lock.

## Trade-offs worth probing
- Lazy (clock-derived) versus timer-driven transitions.
- Counting late results from earlier states versus discarding them.
- One trial at a time versus a small number of probes.

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.

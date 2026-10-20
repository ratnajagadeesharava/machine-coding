# Interviewer notes — 102 Saga Orchestrator

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: safety violation (split brain, lost or divergent committed data), liveness problem (no progress), missed failure case (crash, loss, duplicate, reorder, partition), non-determinism, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Crash after the last step finished but before the saga's COMPLETED record.
- Crash in the middle of compensation: only the not-yet-compensated steps run again, still in reverse order.
- A saga whose log shows a failed step but no compensation started (crash between the failure and the first compensation).
- A restart in the middle of the retries of one step: the attempt counter need not be persisted, but state which behaviour you chose (fresh 3 attempts) and why it is safe.
- Eight threads calling `run` for the same id at once: the steps run exactly once, the losers get RUNNING or the final status.
- COMPENSATION_FAILED is terminal: `run` again returns it without calling anything.

## Likely interviewer questions
- What exactly do you write to the log and when? What does a crash between 'execute' and 'log done' imply for the step?
- Why do steps and compensations have to be idempotent? What is the alternative?
- Why is the failed step not compensated? What if it half-succeeded?
- How do you make `status` correct for a saga another thread is running right now?
- What stops two orchestrators on the same log from running the same saga?
- Why do the sleeps differ between steps (2) and compensations (4)?

## Invariants
- No step is reported done twice and no finished step/compensation is executed again after a restart (at-most-once per log record, at-least-once overall).
- Compensations run in strict reverse order of step completion and only for completed steps.
- A saga ends in exactly one terminal status and never leaves it.
- Two concurrent `run` calls for one saga never execute its steps concurrently.
- The status is a pure function of the log (plus the in-process running guard).

## Common mistakes
- Holding one global lock around `run`, so sagas serialise (the concurrency test then times out or the RUNNING test deadlocks).
- Keeping progress only in memory so a restart repeats everything.
- Logging 'done' before the step ran, or never logging the failure so recovery goes the wrong direction.
- Sleeping after the last failed attempt, or the wrong backoff series for compensation.
- Compensating the failed step, or compensating in forward order.
- Continuing with the remaining compensations after one failed for good.
- Calling `Thread.sleep` instead of the sleeper (Java) / ignoring the injected sleeper (Rust).

## Over-engineering traps
- A saga DSL, annotation scanner or builder framework for steps.
- A workflow engine with a state-machine library and an event bus.
- Async executors and futures per step; the contract is a blocking `run`.

## Distributed-systems discussion
- Temporal / Cadence persist every step of the workflow as event-sourced history and replay it deterministically after a worker crash; AWS Step Functions and Camunda do the same with a managed state store; Uber's Cadence popularised 'durable execution'.
- The saga pattern gives up isolation (ACD, not ACID): other transactions see intermediate states, which is why semantic locks, commutative updates and pivot steps are used. 2PC (problem 101) is the strongly consistent but blocking alternative.
- Exactly-once is not available: the log gives at-least-once execution, so idempotency keys (problem 97) or deduplicating steps are needed in the downstream services.
- Simplified here: a single in-process orchestrator, a perfect log, deterministic retry delays without jitter, no step timeouts, no versioning of the step list across deployments.

## Trade-off questions to ask when the candidate is done
- Orchestration vs choreography: who knows the whole flow, and which one is easier to debug?
- Log before vs after executing the step: what failure does each choice leave you with?
- Backward recovery (compensate) vs forward recovery (retry forever): when is each right?
- What would you page a human for? How does COMPENSATION_FAILED get resolved?
- How would you change the step list of a saga that is mid-flight during a deployment?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.

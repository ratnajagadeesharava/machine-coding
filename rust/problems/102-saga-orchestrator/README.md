# 102 · Saga Orchestrator (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 3 — infrastructure provided: a deterministic network and clock simulation (`SimNet`) runs your nodes; the protocol code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

An order flow touches inventory, payment and shipping, each its own service with its own database; a distributed transaction across them is out of the question. Instead each step has a compensating action (release stock, refund, cancel shipment), and an orchestrator runs the steps in order, retries flaky ones, and on a permanent failure undoes everything that already happened.

Build that orchestrator. It must survive its own process dying at any moment: a new orchestrator started on the same durable log has to pick up the saga where the old one stopped, without re-running finished steps. Many sagas run concurrently on many threads.

**In short:** one orchestrator object with `run(saga_id, steps)` and `status(saga_id)`. You call the steps (which are provided by the caller), call the injected sleeper for back-off, and write progress to the provided log so that a fresh orchestrator can finish a saga that an earlier one left half-done. The tests check the order of step and compensation calls, the exact sleeper values, restart behaviour and thread safety.

## Requirements

1. `run(sagaId, steps)` runs the steps in order on the calling thread and returns the final status. Each step reports failure by returning false (Java: or by throwing; treated the same).
2. A step is attempted at most 3 times. The sleeper is called with 100 before attempt 2 and 200 before attempt 3, and never after the last failed attempt. Three failures fail the saga.
3. After a failed step, the completed steps are compensated in REVERSE order; the failed step itself is not compensated. A compensation is attempted at most 5 times with sleeps 100, 200, 400, 800 between attempts.
4. If a compensation still fails after 5 attempts, the saga ends as COMPENSATION_FAILED and the remaining compensations are NOT run.
5. Progress is written to the `SagaLog`, so a new orchestrator on the same log continues: a finished step is not executed again, a finished compensation is not run again. `status(sagaId)` is derived from the log and is valid right after a restart.
6. `run` on a finished saga returns its final status without calling any step. `run` on a saga another thread is running right now returns RUNNING immediately. An unknown saga is NOT_FOUND; an empty saga is COMPLETED.
7. Different sagas run concurrently and independently; the orchestrator is thread-safe.

## Simulation rules and constraints

- Not a network simulation: the orchestrator is an ordinary object called from several threads (up to 8 in the tests, 400 sagas in total). It must be thread-safe.
- There is no clock: waiting is the injected sleeper (`LongConsumer` / `Fn(u64)`); the tests record the values instead of sleeping. Never call `Thread.sleep` / `thread::sleep` yourself.
- The `SagaLog` (provided) is atomic and durable on return from `append`; a restart is a new orchestrator over the same log. In-memory state is lost on restart, so everything needed to resume must be derivable from the log.
- Different sagas must not block each other; only two calls for the SAME saga id exclude each other.
- Steps and compensations are idempotent per saga id (the tests' steps are), because a crash can happen after a step ran but before it was logged.

## Examples

```
steps: reserve, charge, ship        ship fails every time
run("s1"):  exec reserve, exec charge, exec ship (fail) sleep 100, exec ship (fail) sleep 200, exec ship (fail)
            comp charge, comp reserve   -> COMPENSATED      (ship itself is not compensated)
compensation of charge fails all 5 attempts: sleeps 100,200,400,800 -> COMPENSATION_FAILED, reserve untouched
```

A flaky step that recovers:

```
steps: a, flaky     flaky fails twice and succeeds on its 3rd attempt
run("s1"): exec a, exec flaky (fail) sleep 100, exec flaky (fail) sleep 200, exec flaky (ok)   -> COMPLETED
sleeper calls in order: 100, 200
```

First step fails, nothing to undo:

```
steps: reserve, charge              reserve fails every time
run("s1"): exec reserve x3 (sleeps 100, 200)   -> COMPENSATED, no compensation is called, charge never runs
```

Compensation that needs retries:

```
steps: reserve, charge, ship        ship fails every time; charge's compensation succeeds on its 4th attempt
sleeps in order: 100, 200            (retries of ship)
                 100, 200, 400       (3 failed compensation attempts of charge, then the 4th succeeds)
then comp reserve (ok)               -> COMPENSATED
```

Restart (the log is shared, the in-memory state is not):

```
First orchestrator ran reserve and charge and logged both, then the process died during ship.
A new orchestrator on the same log: run("s1", [reserve, charge, ship]) executes only ship (and, if ship fails for good,
compensates charge then reserve). If the first one died halfway through compensation, only the compensations that are not
yet logged as done are run, still in reverse order. status("s1") is RUNNING until it finishes.
```

Concurrency:

```
Thread 1 is inside run("s1", ...) (a step is executing). Meanwhile on thread 2:
  status("s1")              -> RUNNING
  run("s1", steps)          -> RUNNING immediately, no step is called, thread 1's work is not disturbed
Thread 2 may run "s2" at the same time without waiting for "s1".
```

## Edge cases

- A step fails by returning false; both count as one failed attempt.
- Attempt counting: a step gets at most 3 attempts. The sleeper is called after attempt 1 fails (100) and after attempt 2 fails (200), never after attempt 3. A compensation gets at most 5 attempts with sleeps 100, 200, 400, 800 after attempts 1 to 4, none after attempt 5.
- If the LAST step is the one that fails, all earlier steps are compensated in reverse order. Steps after the failed one were never started and are not compensated.
- If a compensation fails for good, the saga ends as COMPENSATION_FAILED; earlier steps keep their effects (their compensations are not attempted).
- An empty step list gives COMPLETED. `status` of an id the log never saw is NOT_FOUND (also right after a restart). `run` on a finished saga, with any step list, returns the stored final status and calls neither a step nor the sleeper.
- The sleeper is only a callback that records or performs a delay: you pass it the number of milliseconds and keep going.
- Two different saga ids never wait for each other, not even while one of them is sleeping or executing a slow step.

## Glossary

- **Saga**: a business operation made of ordered local steps across services, with no global transaction.
- **Step / compensation**: a step does a piece of work (reserve stock); its compensation undoes it (release stock).
- **Orchestrator**: the object that runs the steps, retries them and triggers compensations.
- **Idempotent**: calling it again for the same saga id has no extra effect; this is why re-running a step that may have already happened is safe.
- **Back-off / sleeper**: waiting longer between successive retries; the wait is performed by the injected sleeper function, not by you.
- **Durable log**: stable storage (`SagaLog`) that survives a restart; progress must be recoverable from it alone.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `Solution.orchestrator(SagaLog, LongConsumer sleeper)`; `SagaStep.execute/compensate` may return false or throw. Rust: `create(log, sleeper)` with `Arc<dyn SagaLog>` and `Arc<dyn Fn(u64)>`; steps signal failure only with `false`; `SagaStatus` variants are `NotFound`, `Running`, `Completed`, `Compensated`, `CompensationFailed`.
- Step retry: max 3 attempts, sleeps (100, 200) between them, no sleep after the last failure. Compensation retry: max 5 attempts, sleeps (100, 200, 400, 800) between them. A sleep is a call to the sleeper with the milliseconds, in order.
- Compensation order is the reverse of the completed steps; the failed step and the not-yet-run later steps are never compensated. A saga whose first step fails is COMPENSATED with no compensation at all.
- Resumption: with any prefix of a saga's log records, `run` with the same step list must finish with the same final status as the uninterrupted run, never repeating a step or compensation that the log shows as finished (a step that had started but whose completion was not logged may run again).
- A finished saga (COMPLETED, COMPENSATED, COMPENSATION_FAILED) is never run again; `run` returns the stored status and calls no step.
- `status` is RUNNING while a saga has been started and not finished (including while another thread is running it), NOT_FOUND for a saga the log has never seen.
- The log record format is yours; it is replayed by `status` and by `run` after a restart.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point for the saga orchestrator. Steps and the log are provided by the tests; `sleeper(ms)` is how
//! you "wait" between retries (the tests record the values instead of sleeping). The traits take `&self`.
//!
//! Rules (the tests check all of them):
//! - Steps run in order. A step is attempted at most 3 times; before attempt 2 and 3 you call the sleeper with 100
//!   and 200. A step that fails 3 times fails the saga. `false` is the failure signal.
//! - Then the steps that completed are compensated in REVERSE order (the failed step is not compensated). A
//!   compensation is attempted at most 5 times, sleeping 100, 200, 400, 800 between attempts. If one still fails the
//!   saga ends as `CompensationFailed` and the remaining compensations are NOT run.
//! - Progress is written to the `SagaLog` so a new orchestrator on the same log continues where the old one stopped:
//!   a finished step is not executed again, a finished compensation is not run again.
//! - `run` on a finished saga returns its final status without calling any step. `run` on a saga that is running
//!   right now (another thread) returns `Running` immediately.
//! - Different sagas run concurrently; the orchestrator must be thread-safe.

use std::sync::Arc;

/// One local transaction of a saga plus its compensating action. Both must be idempotent per saga id
/// (they can be invoked again after a crash).
pub trait SagaStep: Send + Sync {
    fn name(&self) -> String;

    fn execute(&self, saga_id: &str) -> bool;

    fn compensate(&self, saga_id: &str) -> bool;
}

/// Stable storage of the orchestrator (provided by the tests; it survives a restart: a restarted orchestrator is a
/// NEW instance created with the SAME log). Appending is atomic and durable as soon as the call returns.
pub trait SagaLog: Send + Sync {
    fn append(&self, record: &str);

    /// Everything appended so far, oldest first.
    fn records(&self) -> Vec<String>;
}

pub type Log = Arc<dyn SagaLog>;
pub type Sleeper = Arc<dyn Fn(u64) + Send + Sync>;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SagaStatus {
    /// never seen by this log
    NotFound,
    /// started, not finished (possibly being run right now)
    Running,
    /// every step executed
    Completed,
    /// a step failed for good and every earlier step was compensated
    Compensated,
    /// a compensation failed for good: needs a human
    CompensationFailed,
}

pub trait SagaOrchestrator: Send + Sync {
    /// Runs (or continues) the saga to its end on the calling thread and returns the final status.
    fn run(&self, saga_id: &str, steps: &[Arc<dyn SagaStep>]) -> SagaStatus;

    /// The status derived from the log (also valid right after a restart).
    fn status(&self, saga_id: &str) -> SagaStatus;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(_log: Log, _sleeper: Sleeper) -> Option<Arc<dyn SagaOrchestrator>> { ... }
```

## What the tests check

These tests are black-box and deterministic: every node runs inside a seeded simulation (one tick = 1 ms of simulated time, nothing sleeps), messages can be delayed, dropped, duplicated and reordered, nodes can crash and recover, the network can be partitioned, and safety invariants are checked after EVERY simulated millisecond. A failing seed reproduces exactly.

- Steps run in order and complete; an empty saga completes; an unknown saga is NOT_FOUND.
- A failing step triggers exactly three executions followed by compensation of the earlier steps in reverse order, without compensating the failed one or running later steps; a failing first step compensates nothing.
- A flaky step that succeeds on the third attempt completes the saga with sleeps `[100, 200]`; no sleep happens after the third failure; an exception counts as a failure (Java).
- Compensation retries: a stubborn compensation succeeding on its 4th attempt gives sleeps `100,200,100,200,400` overall; one that never succeeds is attempted 5 times (sleeps 100,200,400,800), ends in COMPENSATION_FAILED and the earlier step is not compensated.
- A finished saga is not executed again; independent sagas keep independent statuses.
- Restart: a second orchestrator on the same log reports the right status of finished sagas; crash-injection tests cut the log after every possible number of records, resume, and require the same final status with no duplicated executions and compensations in reverse order.
- Concurrency: 8 threads run 400 sagas (one third doomed) without mixing journals; a saga being run by one thread reports RUNNING from `status` and `run` on other threads.
- More edge cases (random crash points over random failure scripts, 8 threads racing for the same saga id 100 times) in `interviewer/` (see `INTERVIEWER_NOTES.md`).

## Language notes

Steps return only `bool` here, there is no exception path.

## Running

```bash
cargo test -p p102_saga_orchestrator
STRICT=1 cargo test -p p102_saga_orchestrator   # fail instead of skip while unimplemented
cargo test -p p102_saga_orchestrator --features hidden   # interviewer scenarios
cargo play run 102              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`SimNet`, `Rng`, `sut!`): `rust/support`.

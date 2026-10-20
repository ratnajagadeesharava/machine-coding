# 102 · Saga Orchestrator (Java)

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
- Different sagas must not block each other; only two calls for the SAME saga id exclude each other. Do not hold a global lock while a step executes or while sleeping.
- Steps and compensations are idempotent per saga id (the tests' steps are), because a crash can happen after a step ran but before it was logged.

## Examples

```
steps: reserve, charge, ship        ship fails every time
run("s1"):  exec reserve, exec charge, exec ship (fail) sleep 100, exec ship (fail) sleep 200, exec ship (fail)
            comp charge, comp reserve   -> COMPENSATED      (ship itself is not compensated)
compensation of charge fails all 5 attempts: sleeps 100,200,400,800 -> COMPENSATION_FAILED, reserve untouched
```

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `Solution.orchestrator(SagaLog, LongConsumer sleeper)`; `SagaStep.execute/compensate` may return false or throw. Rust: `create(log, sleeper)` with `Arc<dyn SagaLog>` and `Arc<dyn Fn(u64)>`; steps signal failure only with `false`; `SagaStatus` variants are `NotFound`, `Running`, `Completed`, `Compensated`, `CompensationFailed`.
- Step retry: max 3 attempts, sleeps (100, 200) between them, no sleep after the last failure. Compensation retry: max 5 attempts, sleeps (100, 200, 400, 800) between them. A sleep is a call to the sleeper with the milliseconds, in order.
- Compensation order is the reverse of the completed steps; the failed step and the not-yet-run later steps are never compensated. A saga whose first step fails is COMPENSATED with no compensation at all.
- Resumption: with any prefix of a saga's log records, `run` with the same step list must finish with the same final status as the uninterrupted run, never repeating a step or compensation that the log shows as finished (a step that had started but whose completion was not logged may run again).
- A finished saga (COMPLETED, COMPENSATED, COMPENSATION_FAILED) is never run again; `run` returns the stored status and calls no step.
- `status` is RUNNING while a saga has been started and not finished (including while another thread is running it), NOT_FOUND for a saga the log has never seen.
- The log record format is yours; it is replayed by `status` and by `run` after a restart.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.List;

/**
 * Stable storage of the orchestrator (provided by the tests; it survives a restart: a restarted orchestrator is a NEW
 * instance created with the SAME log). Appending is atomic and durable as soon as the call returns.
 */
public interface SagaLog {

    void append(String record);

    /** Everything appended so far, oldest first. */
    List<String> records();
}

package contract;

import java.util.List;

/**
 * Runs sagas: a sequence of steps, each with a compensation.
 *
 * <p>Rules (the tests check all of them):
 * <ul>
 *   <li>Steps run in order. A step is attempted at most 3 times; before attempt 2 and 3 the orchestrator calls the
 *       sleeper with 100 and 200 (ms). A step that fails 3 times fails the saga.</li>
 *   <li>Then the steps that completed are compensated in REVERSE order (the failed step is not compensated). A
 *       compensation is attempted at most 5 times, sleeping 100, 200, 400, 800 between attempts. If one still fails,
 *       the saga ends as COMPENSATION_FAILED and the remaining compensations are NOT run.</li>
 *   <li>Progress is written to the {@link SagaLog} so that a new orchestrator on the same log continues where the
 *       old one stopped: a finished step is not executed again, a finished compensation is not run again.</li>
 *   <li>{@code run} on a saga that already finished returns its final status without calling any step. {@code run}
 *       on a saga that is running right now (another thread) returns RUNNING immediately.</li>
 *   <li>Different sagas run concurrently; the orchestrator must be thread-safe.</li>
 * </ul>
 */
public interface SagaOrchestrator {

    /** Runs (or continues) the saga to its end on the calling thread and returns the final status. */
    SagaStatus run(String sagaId, List<SagaStep> steps);

    /** The status derived from the log (also valid right after a restart). */
    SagaStatus status(String sagaId);
}

package contract;

public enum SagaStatus {
    /** never seen by this log */
    NOT_FOUND,
    /** started, not finished (possibly being run right now) */
    RUNNING,
    /** every step executed */
    COMPLETED,
    /** a step failed for good and every earlier step was compensated */
    COMPENSATED,
    /** a compensation failed for good: needs a human */
    COMPENSATION_FAILED
}

package contract;

/**
 * One local transaction of a saga plus its compensating action (provided by the tests).
 * Both methods report failure by returning {@code false} OR by throwing: the orchestrator treats the two the same.
 * Both must be idempotent per saga id (they can be invoked again after a crash).
 */
public interface SagaStep {

    String name();

    boolean execute(String sagaId);

    boolean compensate(String sagaId);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    /**
 * Wiring point between the test contract and YOUR design.
 * Create your own classes anywhere under src/main/java and return them from here.
 */
    public static SagaOrchestrator orchestrator(SagaLog log, LongConsumer sleeper)
}
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

`ConcurrentHashMap.newKeySet()` / `putIfAbsent` or `compute` gives a cheap per-saga 'currently running' guard without a global lock; remember that `records()` returns a copy.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/102-saga-orchestrator test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`SimNet`, `FakeClock`, `Candidate`): `java/testkit`.

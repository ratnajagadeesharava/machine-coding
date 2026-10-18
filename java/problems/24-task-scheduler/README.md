# 24 · Task Scheduler (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 3 — infrastructure provided. Test utilities for threads/clocks exist; business/domain code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Build an in-process task scheduler: callers hand it units of work to run **later** (once, or repeatedly), may
cancel them, and the host application periodically asks the scheduler to run whatever has become due.

There are no worker threads of your own: time is driven by an injected clock and execution is driven by the caller,
which makes the whole thing deterministic and testable.

## Requirements

1. A one-shot task is scheduled with a delay and runs once when its due time has been reached.
2. A recurring task is scheduled with an initial delay and a period. It keeps running until it is cancelled.
3. The caller triggers execution with a **run-due** call. Every task whose due time has been reached runs, on the calling thread, before the call returns. The call reports how many task executions it performed.
4. Tasks that are due run in order of **due time**; tasks with the same due time run in the order they were **submitted**.
5. A recurring task is **fixed-rate**: its schedule is anchored on the planned due times, not on when the previous run happened to finish or when the host got around to calling run-due. If the host was late, every missed occurrence still runs.
6. Tasks can be cancelled by id. Cancelling a task that is due but has not run yet must stop it from running.
7. A task that fails must not prevent other tasks from running, and a failing recurring task keeps its schedule.
8. Tasks are ordinary code and may use the scheduler themselves: schedule more work, or cancel other tasks (or themselves).
9. Scheduling, cancelling and running may happen from many threads at once.

## Constraints

- Time comes only from the clock given to the factory; never sleep or read the wall clock.
- Up to ~100,000 pending tasks; a run-due call that has nothing due must be cheap (do not scan every pending task).
- A task must never be executed twice for the same occurrence, no matter how many threads call run-due.
- A task that uses the scheduler from inside its own execution must not deadlock the scheduler.

## Examples

```
now = 1000
scheduleOnce(100, A)                 → id1        (due 1100)
scheduleRecurring(100, 10, R)        → id2        (due 1100, 1110, 1120, ...)
runDue() at 1000                     → 0
runDue() at 1135                     → 5          (A, R@1100, R@1110, R@1120, R@1130 — in due-time order)
runDue() at 1140                     → 1          (R@1140)
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- Times are epoch milliseconds from the injected clock. A task scheduled with `delay` at time `now` is due at `now + delay`; a due time `≤` the current time means "due" (delay 0 is due immediately).
- `scheduleOnce(delay, task)` returns a new unique id. `scheduleRecurring(initialDelay, period, task)` returns a new unique id, or empty if `period ≤ 0`. Its occurrences are due at `now + initialDelay + k × period` for `k = 0, 1, 2, …` (next due = previous due + period).
- `runDue()` reads the clock **once** at the start; it executes every occurrence with due time `≤` that time, in order of `(due time, submission order)` where submission order is the order in which the task was scheduled (all occurrences of a recurring task share its submission order). For a recurring task **all** missed occurrences run (catch-up), interleaved with other tasks by due time. It returns the number of executions performed by that call (a run that throws still counts as an execution). It returns 0 when nothing is due.
- Tasks scheduled from inside a running task are ordinary tasks: if their due time is `≤` the time `runDue` read at its start, they run in the same call (in their due-time order); otherwise later.
- `cancel(id)` returns `true` if it stopped a task that was still going to run (a one-shot that has not started, or a recurring task that was not yet cancelled — including from inside its own execution, which stops all of its later occurrences, including missed ones not yet run in the current call). It returns `false` for unknown ids, one-shots that already started or finished, and tasks already cancelled.
- A task that throws (panics) does not stop `runDue` or other tasks; a recurring task that throws still has its later occurrences run.
- Tasks run on the thread that called `runDue`. Two threads calling `runDue` concurrently must not run the same occurrence twice.

## Test entry point (Java)

The tests drive your system through the thin, primitive-typed contract below. It is **not** your domain
model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in
`solution/Solution.java`.

```java
package contract;

import java.util.Optional;

/** Thin test entry point. Your domain model lives behind it. Must be thread-safe. */
public interface TaskSchedulerApi {

    /** Runs the task once, when its due time (now + delay) has been reached by a runDue call. */
    String scheduleOnce(long delayMillis, Runnable task);

    /** Empty if periodMillis <= 0. Fixed-rate: occurrence k is due at now + initialDelay + k * period. */
    Optional<String> scheduleRecurring(long initialDelayMillis, long periodMillis, Runnable task);

    /** True if it stopped a task that was still going to run. */
    boolean cancel(String id);

    /** Runs everything due on the calling thread; returns the number of executions performed. */
    int runDue();
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    TaskSchedulerApi create(Clock clock)
}
```

## Language notes

`Clock` is `java.time.Clock` (use `clock.millis()`). The task type is `Runnable`. Exceptions thrown by tasks must be contained.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/24-task-scheduler test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

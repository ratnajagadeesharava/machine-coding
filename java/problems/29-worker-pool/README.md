# 29 · Worker Pool (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 3 — infrastructure provided. Test utilities for threads/clocks exist; business/domain code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

**What you are building.** A backend service receives bursts of small jobs (resize an image, call a slow API,
compute a checksum). Starting a brand-new thread for every job is wasteful and, under a burst, unbounded. You build
a **worker pool**: a fixed set of long-lived worker threads that run submitted jobs, a bounded waiting line for the
jobs that arrive while every worker is busy, an immediate "no, I am full" answer when even the line is full, and a
graceful shutdown that never throws away work the pool has already agreed to do. Callers get a handle back so they
can collect each job's result (or its failure) whenever they like.

### Terms used below

- **Job**: a piece of work that, when run, produces one number (it may also fail).
- **Worker**: one of the pool's long-lived threads. It runs one job at a time.
- **Queue capacity**: the maximum number of *accepted* jobs that may be *waiting* (not yet started) at once.
- **Accepted**: the pool took the job and promises to run it. **Rejected**: the pool refused it; the job never runs.
- **Outcome**: what a caller eventually receives for an accepted job: the number, or the failure.

## Requirements

1. The pool is created with a number of **workers** (at least 1) and a **queue capacity** (at least 0): how many accepted jobs may wait for a free worker.
2. A caller **submits** a job (a computation that yields a number) and gets back something it can use to obtain the outcome later: the number, or the failure. Each caller gets the outcome of *its own* job, even when many threads submit at once.
3. At most `workers` jobs run **at the same time**, and when enough jobs are available that many really do run in parallel (3 workers and 10 queued-up long jobs: exactly 3 are running, not 2).
4. Submitting never blocks the caller. If every worker is busy and the queue already holds `queueCapacity` waiting jobs, the submission is **rejected** and the job never runs.
5. Waiting jobs start in the order they were submitted (first submitted, first started).
6. A job that fails (throws or panics) reports its failure to its own caller only and must not harm the pool: the worker keeps serving later jobs.
7. **Shutdown** stops the pool from accepting new jobs (every later submission is rejected) but **every job already accepted — running or queued — still runs to completion**. `shutdown` does not wait for them. Calling it again has no effect.
8. A caller can **wait for termination** for a bounded time: it learns (true/false) whether, after shutdown, the pool has finished every accepted job.
9. Jobs may submit further jobs to the same pool.

## Constraints

- Thread-safe: any number of threads submit concurrently.
- Accepted jobs are started as soon as a worker is free; a caller must not need to poke the pool to make progress.

## Examples

```
workers 2, queue capacity 3
five long jobs   → accepted (2 run, 3 wait)
a sixth job      → rejected immediately
the first ends   → the oldest waiting job starts; shortly afterwards a new submission is accepted again
shutdown         → new submissions rejected, the five accepted jobs still complete
```

Capacity arithmetic (workers 1, queue capacity 1):

| Step | Call | Result |
|---|---|---|
| 1 | submit A (long) | accepted, A starts running |
| 2 | submit B | accepted, B waits (queue now full) |
| 3 | submit C | **rejected**, C never runs |
| 4 | A and B finish | their outcomes are available |
| 5 | submit D (retry briefly if still rejected) | accepted, outcome `4` if D yields `4` |

Failure isolation (1 worker): five jobs that each fail with "boom", then a job yielding `7`. Each failing caller sees
its own "boom" failure; the sixth caller still receives `7`.

Shutdown and termination (workers 1, queue capacity 5, job 0 running, jobs 1-3 waiting, all blocked):

| Step | Call | Result |
|---|---|---|
| 1 | `shutdown` | returns immediately |
| 2 | submit another job | rejected |
| 3 | `awaitTermination(100)` | `false` (accepted jobs still unfinished) |
| 4 | let the jobs finish, `awaitTermination(10000)` | `true`; all four outcomes are available |

## Edge cases

- Queue capacity 0: a job is accepted only if a worker is free right now to take it; nothing ever waits.
- `awaitTermination` on a pool that was never shut down returns `false` after (about) the full timeout and does **not** shut the pool down; the pool still accepts jobs afterwards.
- `awaitTermination` after `shutdown` on a pool with no unfinished jobs returns `true`, however often you ask.
- A rejected submission has no effect at all: the job never runs and the pool's state is unchanged.

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- Java: `submit` returns a future for the job's value and throws `RejectedExecutionException` when rejected (queue full while all workers busy, or after shutdown). A job that throws completes its future exceptionally (the cause is the thrown exception). Rust: `submit` returns `Err(Rejected)` or a receiver that yields exactly one `Ok(value)` or `Err(message)` (a panicking job produces `Err` with the panic message).
- A rejected job is never run and leaves the pool unchanged.
- Capacity is measured in waiting jobs: with `w` workers and capacity `c`, `w + c` jobs can be accepted while none has finished. A job occupies a worker or queue slot from acceptance until it has been taken over by a worker / finished; the slot of a finished job becomes reusable shortly after the outcome is available (callers that need to resubmit should retry briefly).
- `queueCapacity == 0` means a job is accepted only if a worker is free to take it.
- `shutdown()` returns immediately. After it, all submissions are rejected. Accepted jobs run to completion.
- `awaitTermination(timeoutMs)` returns `true` when `shutdown()` has been called and every accepted job has finished, `false` if that did not happen within the timeout (it also returns `false` if `shutdown()` was never called). It does not itself shut the pool down.
- Waiting jobs start in submission order (strictly, when there is one worker; with several workers jobs are *taken* from the queue in that order).
- Java: `awaitTermination` may throw `InterruptedException`.

## Test entry point (Java)

The tests drive your system through the thin, primitive-typed contract below. It is **not** your domain
model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in
`solution/Solution.java`.

```java
package contract;

import java.util.concurrent.Callable;
import java.util.concurrent.CompletableFuture;
import java.util.concurrent.RejectedExecutionException;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface WorkerPoolApi {

    /** Never blocks. Throws RejectedExecutionException when full or shut down; a failing job completes the future exceptionally. */
    CompletableFuture<Long> submit(Callable<Long> job) throws RejectedExecutionException;

    /** Stops accepting jobs; accepted jobs still complete. Returns immediately; idempotent. */
    void shutdown();

    /** True iff shutdown was called and all accepted jobs finished within the timeout. */
    boolean awaitTermination(long timeoutMs) throws InterruptedException;
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    WorkerPoolApi create(int workers, int queueCapacity)
}
```

## Language notes

The job is a `Callable<Long>`. Tests wait for jobs with latches; do not use sleeping as a synchronisation mechanism in your implementation either.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/29-worker-pool test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

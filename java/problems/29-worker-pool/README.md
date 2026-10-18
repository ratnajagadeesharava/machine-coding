# 29 · Worker Pool (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 3 — infrastructure provided. Test utilities for threads/clocks exist; business/domain code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

A service receives bursts of small jobs. Starting a thread per job is wasteful and unbounded. Build a pool of long-lived
workers that executes submitted jobs, queues a limited number of them while all workers are busy, tells callers
immediately when it cannot take more, and can be shut down gracefully without dropping accepted work.

## Requirements

1. The pool is created with a number of **workers** (at least 1) and a **queue capacity** (at least 0): how many accepted jobs may wait for a free worker.
2. A caller **submits** a job (a computation that yields a number) and gets back something it can use to obtain the outcome later: the number, or the failure.
3. At most `workers` jobs run **at the same time**, and when enough jobs are available that many really do run in parallel.
4. Submitting never blocks the caller. If every worker is busy and the queue already holds `queueCapacity` waiting jobs, the submission is **rejected** and the job never runs.
5. Waiting jobs start in the order they were submitted.
6. A job that fails (throws or panics) reports its failure to its own caller and must not harm the pool: the worker keeps serving jobs.
7. **Shutdown** stops the pool from accepting new jobs (they are rejected) but **every job already accepted — running or queued — still runs to completion**. `shutdown` does not wait. Calling it again has no effect.
8. A caller can **wait for termination** for a bounded time: it learns whether the pool has finished everything after shutdown.
9. Jobs may submit further jobs to the same pool.

## Constraints

- Thread-safe: any number of threads submit concurrently.
- Waiting workers must not burn CPU.
- A job's own lock-holding or blocking must never be done under a lock that other submitters or workers need.
- Do not rely on sleeping to coordinate; accepted jobs are started as soon as a worker is free.

## Examples

```
workers 2, queue capacity 3
five long jobs   → accepted (2 run, 3 wait)
a sixth job      → rejected immediately
the first ends   → the oldest waiting job starts; shortly afterwards a new submission is accepted again
shutdown         → new submissions rejected, the five accepted jobs still complete
```

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

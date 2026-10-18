# Interviewer notes — 29 Worker Pool

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- Queue capacity 0: only a free worker may accept a job; two submissions back to back with two idle workers must both succeed.
- Failing jobs interleaved with good ones: workers survive, parallelism stays at `workers`.
- A job submitting another job (reentrancy) must not deadlock or be rejected spuriously when there is room.
- Shutdown on an idle pool: terminates immediately, later submissions rejected.
- Results delivered to the right caller under heavy concurrent submission.

## Likely interviewer questions
- What are the shared structures (queue, state flag, counts) and which lock protects what?
- How do you decide "reject" atomically with "enqueue" so two submitters cannot both take the last slot?
- How do idle workers wait? What wakes them for new work and for shutdown?
- What exactly does shutdown guarantee? What do you do with jobs that are still queued?
- Where is a job's exception caught, and who sees it? What if the job throws an `Error`?
- How do you know when the pool has terminated (last worker exits)? How does `awaitTermination` wait?

## Invariants
- Running jobs <= workers; waiting jobs <= queue capacity.
- A job is run at most once, and every accepted job runs exactly once.
- After shutdown, accepted jobs still complete and nothing new is accepted.
- The pool never loses a worker because of a failing job.

## Common mistakes
- Checking "queue full" and enqueueing in two separate steps (over-acceptance under contention).
- Counting a job that has been accepted but not yet picked up as free capacity, or double-counting running jobs.
- Catching exceptions around the worker loop instead of around each job (the worker dies).
- Holding the pool lock while running the job; submitting from inside a job then deadlocks.
- Shutdown that drops the queue, or workers that exit as soon as the flag is set while the queue is not empty.
- `awaitTermination` that polls with `sleep` or ignores the timeout; returns true before shutdown.
- Spawning a thread per job "because the pool is full".

## Over-engineering traps
- `Task`, `TaskQueue`, `WorkerThread`, `PoolState` hierarchy, a `RejectionPolicy` strategy and a `ThreadFactory` before the basic pool works.
- Wrapping everything in your own `Future` implementation when a completable future / channel does the job.

## Concurrency discussion
- Simplest correct design: one lock, one condition for "work available", a deque, an accepted-jobs counter, and worker threads that loop `take next job or exit on shutdown`.
- Alternatives: a bounded blocking queue (non-blocking `offer`) — but then capacity 0 and "free worker" accounting need care.
- Termination: a counter of live workers plus a condition, or joining the worker threads.

## Trade-offs worth probing
- One lock for everything versus lock-free queue plus atomics.
- Pre-started workers versus lazily started.
- Rejecting versus blocking the submitter.

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.

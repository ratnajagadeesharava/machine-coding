# Interviewer notes — 24 Task Scheduler

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- Same due time between a recurring and a one-shot task: submission order decides.
- A recurring task with a very small period and a large catch-up (e.g. period 1 over 1000 ms).
- A one-shot cancelling itself (already started → `false`).
- A recurring task that cancels itself part-way through catch-up.
- Calling `runDue` twice in a row; the second returns 0.
- A throwing one-shot still counts as an execution.

## Likely interviewer questions
- What is the data structure for pending work (heap, sorted map, timing wheel)? Cost of schedule / cancel / run-due?
- How do you cancel in O(log n) or O(1) — remove eagerly, or mark and skip lazily? What does the lazy approach cost in memory?
- Which lock is held while a task runs? Why must the answer be "none"?
- What does "fixed-rate with catch-up" do to a task that is slower than its period — and is that what you want in production (skip? coalesce?)
- How do you give a total order to equal due times?
- How would you test time-dependent code (injected clock) — and what changes when a real timer thread is involved?

## Invariants
- Each occurrence is executed at most once, by exactly one thread.
- Occurrences of one task run in due-time order.
- A cancelled task never starts a new execution after `cancel` returned `true`.
- No lock is held while user code runs.

## Common mistakes
- Holding the scheduler lock while running a task (re-entrant schedule/cancel deadlocks or throws).
- Recomputing the next due time as `now + period` (drift) instead of `previous due + period`.
- Running only one occurrence of a late recurring task.
- Popping a task, running it, then re-queuing it without checking whether it was cancelled meanwhile.
- Ordering by due time only, so equal-time tasks run in arbitrary order (heap instability).
- An exception from one task escaping `runDue` and abandoning the rest (or killing the recurring schedule).
- Snapshotting the due set up front and so running a task that another task cancelled a moment earlier.
- A recurring task that is "in flight" on one thread being popped again by another thread.

## Over-engineering traps
- `Task`, `OneShotTask`, `RecurringTask`, `TaskFactory`, `SchedulingStrategy`, `ExecutorAdapter`, listeners and a full state machine for what is a priority queue plus a map.
- Spawning a thread or `ScheduledExecutorService` underneath (the clock is injected; time must be driven by the caller).

## Scalability / concurrency discussion
- One mutex around a binary heap / ordered map is enough; the discipline is "lock, pop one due entry, unlock, run, lock, re-queue".
- Timing wheels / hierarchical wheels when schedules are huge and mostly short.
- Sharding by task id when schedule/cancel throughput dominates.

## Trade-offs worth probing
- Eager removal on cancel (needs a handle into the structure) vs tombstones.
- Re-queue the next occurrence before or after running the current one (concurrent runDue behaviour differs).

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.

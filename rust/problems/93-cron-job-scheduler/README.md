# 93 · Cron Job Scheduler (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 75 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Implement the engine of a small job scheduler. Operators define named jobs with a cron-like schedule and a retry policy.
The scheduler does not run on its own thread: something (a ticker, a test) calls `runDue()` whenever it likes, and the engine
executes everything that is due at that moment. Jobs fail now and then; a failing job is retried with growing delays and after
too many failures it goes to a dead-letter list. If the process was down for a while, the job must not suddenly run a hundred
times to catch up. Several callers may poll at the same time and no execution may happen twice.

## Requirements

1. `define(name, cron, job, maxAttempts, initialBackoffMs, maxBackoffMs)` registers a job. `cron` has THREE fields `minute hour weekday` (weekday 0-6, 0 = Sunday). Each field is `*`, `*/n`, a number, a range `a-b` or a comma list of those.
2. The first slot is the first matching minute **strictly after** the minute of the definition. Invalid definitions (duplicate name, bad cron, `maxAttempts < 1`, backoff <= 0) are refused.
3. `runDue()` runs, on the calling thread, everything due now and returns the number of executions (attempts). Order: by due time, then by definition order. Each job executes at most once per call.
4. A failed run is retried up to `maxAttempts` attempts in total; after failure number k the next attempt is due `min(initialBackoffMs * 2^(k-1), maxBackoffMs)` later. A job fails by throwing; failures never escape `runDue` or stop other jobs.
5. If several cron slots were missed, only the LATEST matching minute not after now runs; the earlier ones are neither run nor recorded. After a slot ran or was skipped, the next slot is the first matching minute after the current minute.
6. If a slot becomes due while the job is still waiting for a retry, that slot is SKIPPED (recorded, not run). Slots never overlap.
7. `pause(name)`: no runs, no retries, `nextSlot` is empty. `resume(name)`: the next slot is recomputed from now. `remove(name)` deletes the job.
8. `nextSlot(name)` is the epoch minute of the next slot (empty if paused). `history(name)` lists `slot:attempt:OUTCOME` entries, oldest first. `deadLetters()` lists `name:slot` of jobs that ran out of attempts, in the order they died.

## Constraints

- Several threads may call `runDue()` (and `pause`/`resume`/`define`) at the same time. An attempt must never run twice: 8 threads polling after each of 50 minute ticks execute the job exactly 50 times in total.
- Time comes from the injected clock; "minute" is `epochMillis / 60_000` in UTC and epoch minute 0 is Thursday 1970-01-01 00:00 (so weekday of minute m is `(4 + m/1440) % 7`, 0 = Sunday).
- Jobs run on the polling thread. Do not hold a global lock while running user code if you can avoid it, but a job must not run twice if two callers race.
- Scale: 1,000 jobs polled every simulated minute over a day (1,440 polls), and a 3-week outage with a `* * *` job must produce exactly one run, quickly: finding the next slot must not iterate minute by minute for weeks if you can compute it directly (a bounded scan of a few days is acceptable).
- No real sleeping, no background threads.

## Examples

```
clock = minute 0.  define("j", "*/5 * *", job, maxAttempts=1, ...)   nextSlot = 5
runDue() at minute 4 -> 0;  at minute 5 -> 1;  again at minute 5 -> 0;  at minute 10 -> 1
history = ["5:1:OK", "10:1:OK"],  nextSlot = 15

flaky job, maxAttempts 3, backoff 1s..60s, slot 10:
10:1:FAILED, (1s later) 10:2:FAILED, (2s later) 10:3:OK
```

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `JobService` and the `Job` functional interface (`void run() throws Exception`); `IllegalArgumentException` for every invalid definition and for unknown names. Rust: `JobService` trait, `JobFn = Arc<dyn Fn() -> Result<(), String> + Send + Sync>` (failure is `Err`), `JobError::{InvalidDefinition, UnknownJob}`, `next_slot` returns `Jobs<Option<u64>>`.
- Cron validation: wrong number of fields, minute > 59, hour > 23, weekday > 6, `*/0`, and a reversed range such as `5-1` are all invalid. `*/n` means every n-th value starting from the field's minimum (`*/5` minutes -> 0,5,10,...).
- Weekday numbers: epoch minute 0 is a Thursday, `0 0 0` first matches Sunday 1970-01-04 00:00 = minute 4,320; `0 12 1` first matches Monday 1970-01-05 12:00 = minute 4 * 1,440 + 720 = 6,480.
- History entries: `OK`, `FAILED` (will be retried), `DEAD` (last attempt failed, also added to the dead letters), `SKIPPED` (attempt number 0). The slot in an entry is the minute the slot was scheduled for, e.g. `15:1:OK` after a run at minute 17 for `*/5`.
- Retries have priority over a new slot; the retry's slot number stays the original slot (`5:2:OK` after `10:0:SKIPPED`). After the slot finishes (OK or DEAD) the next slot is the first match after the current minute.
- Ordering in one `runDue` call is by due time ascending, ties by definition order; due time is the retry time, or the next slot's time.
- A job whose retry is not yet due is not run; the number returned counts attempts including failed ones. `runDue` before the first slot returns 0.
- Paused jobs keep their retry state frozen; `resume` recomputes the next slot as the first matching minute strictly after the current minute. After `remove`, `history`/`nextSlot` for that name are errors.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Time comes from the clock handed to `create`. A "minute" is epoch milliseconds / 60,000 (UTC); epoch minute 0 is
//! Thursday 1970-01-01 00:00. Nothing runs by itself: the caller drives the scheduler by calling `run_due`.

use std::sync::Arc;

/// The work a job does (provided by the tests). A job fails by returning `Err`.
pub type JobFn = Arc<dyn Fn() -> Result<(), String> + Send + Sync>;

#[derive(Debug, PartialEq, Eq)]
pub enum JobError {
    /// duplicate name, bad cron expression, max_attempts < 1, backoff of 0
    InvalidDefinition,
    UnknownJob,
}

pub type Jobs<T> = Result<T, JobError>;

pub trait JobService: Send + Sync {
    /// Defines a job. `cron` has THREE fields "minute hour weekday" (weekday 0-6, 0 = Sunday); each field is `*`,
    /// `*/n`, a number, a range `a-b` or a comma list of those. The first run is the first matching minute strictly
    /// after the minute of the definition. A failing run is retried up to `max_attempts` attempts in total; after
    /// failure number k (k = 1, 2, ...) the next attempt is due `min(initial_backoff_ms * 2^(k-1), max_backoff_ms)`
    /// later.
    fn define(
        &self,
        name: &str,
        cron: &str,
        job: JobFn,
        max_attempts: u32,
        initial_backoff_ms: u64,
        max_backoff_ms: u64,
    ) -> Jobs<()>;

    /// Runs, on the calling thread, everything that is due now and returns the number of executions (attempts) made.
    /// Order: by due time, then by definition order. Per job at most ONE execution per call, and
    /// * a retry that is due runs (attempt k+1 of the same slot);
    /// * otherwise if the next cron slot is due it runs, for the LATEST matching minute not after now (missed earlier
    ///   minutes are not run and not recorded);
    /// * if a cron slot is due while the job is still waiting for a retry, the slot is SKIPPED (recorded, not run);
    ///   no two slots overlap.
    ///
    /// After a slot ran or was skipped, the next slot is the first matching minute after the current minute.
    /// Failures never escape and never stop other jobs. Concurrent callers must never execute the same attempt twice.
    fn run_due(&self) -> usize;

    /// Paused jobs neither run nor retry.
    fn pause(&self, name: &str) -> Jobs<()>;

    /// The next slot is recomputed from now (first matching minute strictly after the current minute).
    fn resume(&self, name: &str) -> Jobs<()>;

    fn remove(&self, name: &str) -> Jobs<()>;

    /// Epoch minute of the next cron slot; `None` if the job is paused.
    fn next_slot(&self, name: &str) -> Jobs<Option<u64>>;

    /// Everything that happened for the job, oldest first: "slot:attempt:OUTCOME" with OUTCOME one of OK, FAILED
    /// (will be retried), DEAD (last attempt failed), SKIPPED (attempt is 0).
    fn history(&self, name: &str) -> Jobs<Vec<String>>;

    /// Jobs that ran out of attempts, as "name:slot", in the order they died.
    fn dead_letters(&self) -> Vec<String>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(_clock: Arc<dyn Clock>) -> Option<Arc<dyn JobService>> { ... }
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Time-dependent behaviour uses an injectable fake clock, never real sleeping. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- A `*/5` job: nothing before minute 5, one run at 5, nothing twice, next at 10; history and `nextSlot`.
- Not run in the minute of the definition (`* * *` first slot is minute 1).
- A daily job at 09:30 and weekday handling with Sunday = 0 (`0 12 1` Monday and `0 0 0` Sunday from the Thursday epoch).
- Lists and ranges (`0,30 8-9 *`) across 08:00, 08:30, 09:00, 09:30 and the wrap to the next day.
- Rejected definitions: duplicate, 2 fields, 60, 24, weekday 7, `*/0`, `5-1`, 0 attempts, 0 backoff; unknown job name.
- 17 minutes of downtime for `*/5` run once for slot 15; the others forgotten; next slot 20.
- Exponential backoff timing to the millisecond (1s then 2s), success on the third attempt, no dead letters.
- Capped backoff (1s, 2s, 3s, 3s, 3s) for six attempts and `60:6:DEAD`.
- Giving up (`DEAD`, dead letter `j:10`) while the next slot still runs; skipped slot while retrying (`10:0:SKIPPED` between failed and successful retry).
- Ordering by due time then definition order; a failing job does not stop others and counts as an execution.
- Pause, resume and remove; 8 threads calling `runDue` after each of 50 ticks execute exactly 50 times; 100 repeated races for a failing-then-OK job (`1:1:FAILED`, `1:2:OK`).
- More edge cases (1,000 jobs over a day, a 3-week outage) are in `interviewer/`.

## Language notes

Store fields as `[bool; 60]`, `[bool; 24]`, `[bool; 7]` or bitmasks (`u64`, `u32`, `u8`); keep jobs in a `Vec` with an index for definition order. Take the `JobFn` out as an `Arc` clone while holding the lock, drop the guard, call it, then re-lock to record the outcome; use `saturating_mul`/`checked_shl` for the backoff.

## Running

```bash
cargo test -p p93_cron_job_scheduler
STRICT=1 cargo test -p p93_cron_job_scheduler   # fail instead of skip while unimplemented
cargo test -p p93_cron_job_scheduler --features hidden   # interviewer scenarios
cargo play run 93              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`FakeClock`, `sut!`): `rust/support`.

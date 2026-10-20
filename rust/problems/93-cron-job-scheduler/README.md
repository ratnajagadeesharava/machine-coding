# 93 · Cron Job Scheduler (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 75 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

You are building the engine of a small job scheduler, the kind that runs nightly reports or periodic clean-ups. Operators
define named jobs, each with a cron-style schedule (`minute hour weekday`) and a retry policy (how many attempts, and how long to
wait between them). Nothing runs by itself: whoever owns the clock (a ticker thread in production, a test here) calls
`run_due()`, and the engine executes whatever is due at that instant and reports how many executions it made. Jobs may throw;
a failing job is retried with growing delays and, when it runs out of attempts, lands on a dead-letter list. If the process
was down for hours, a job must run once for the latest missed slot, not once per missed slot. Several threads may poll at the
same time and an attempt must never run twice.

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
9. The three cron fields must ALL match for a minute to be a slot: `minute` in 0-59, `hour` in 0-23, `weekday` in 0-6. `30 9 *` is every day at 09:30 UTC; `0 12 1` is every Monday at 12:00 UTC; `*/15 * *` is minutes 0, 15, 30, 45 of every hour of every day.
10. A comma list may combine the other forms (`0,30 8-9 *` = minutes 0 and 30 of hours 8 and 9, i.e. 08:00, 08:30, 09:00, 09:30). A range `a-b` is inclusive on both ends.
11. A job's `history` is never trimmed by retries: every attempt (`FAILED`, `OK`, `DEAD`) and every `SKIPPED` slot is one entry, oldest first. `dead_letters()` spans all jobs, in the order jobs died.
12. `run_due()` called before any slot or retry is due returns 0 and records nothing. `SKIPPED` entries are not executions and are not counted in the return value; failed attempts are.

## Constraints

- Several threads may call `runDue()` (and `pause`/`resume`/`define`) at the same time. An attempt must never run twice: 8 threads polling after each of 50 minute ticks execute the job exactly 50 times in total.
- Time comes from the injected clock; "minute" is `epochMillis / 60_000` in UTC and epoch minute 0 is Thursday 1970-01-01 00:00 (so weekday of minute m is `(4 + m/1440) % 7`, 0 = Sunday).
- Jobs run on the polling thread. A job must not run twice if two callers race.
- Scale: 1,000 jobs polled every simulated minute over a day (1,440 polls), and a 3-week outage with a `* * *` job must produce exactly one run, quickly.
- No real sleeping, no background threads.

## Glossary

- **Cron expression**: three space-separated fields `minute hour weekday` describing which minutes a job should run at (weekday 0 = Sunday ... 6 = Saturday). All times are UTC.
- **Slot**: one scheduled minute of a job, identified by its epoch minute (`epoch_millis / 60000`). A slot may need several attempts.
- **Attempt**: one execution of the job's code. `max_attempts` counts them all, including the first.
- **Backoff**: the wait before the next attempt after a failure: `min(initial_backoff_ms * 2^(k-1), max_backoff_ms)` after the k-th failure.
- **Dead letter**: a slot whose attempts were all used up and failed; recorded as `name:slot`.
- **Skipped**: a slot that became due while the job was still waiting for a retry of an earlier slot; it is recorded but never run.

## Examples

**Example 1: a simple recurring job** (clock starts at minute 0)

```
define("j", "*/5 * *", job, max_attempts=1, initial_backoff_ms=1000, max_backoff_ms=1000)
next_slot("j")            -> 5          // strictly after the minute of definition
run_due() at minute 4     -> 0
run_due() at minute 5     -> 1          // job ran
run_due() at minute 5     -> 0          // nothing twice
run_due() at minute 10    -> 1
history("j")             -> ["5:1:OK", "10:1:OK"]
next_slot("j")            -> 15
```

**Example 2: retries with exponential backoff** (`*/10 * *`, max_attempts 5, backoff 1 s .. 60 s, the job fails twice and then works)

```
minute 10        run_due() -> 1   history: 10:1:FAILED      (retry due 1 s later)
+999 ms          run_due() -> 0
+1 ms (1 s)      run_due() -> 1   history: 10:2:FAILED      (retry due 2 s later)
+1 999 ms        run_due() -> 0
+1 ms (2 s)      run_due() -> 1   history: 10:3:OK
next_slot -> 20,  dead_letters() -> []
```
With `max_attempts 6`, backoff 1 s .. 3 000 ms and a job that always fails, the gaps between attempts are 1 s, 2 s, 3 s, 3 s, 3 s. With `max_attempts 3`, backoff 1 s and a job that fails every time, the third failure is recorded as `10:3:DEAD`, `dead_letters()` is `["j:10"]`, and the job still runs again at its next slot (20).

**Example 3: downtime and skipped slots**

```
*/5 job, nobody calls run_due() until minute 17:
run_due() -> 1;  history = ["15:1:OK"];  next_slot -> 20      // only the latest slot (15); 5 and 10 are forgotten

*/5 job, max_attempts 3, backoff 8 min, fails once:
minute 5   run_due() -> 1   5:1:FAILED   (retry due at minute 13)
minute 10  run_due() -> 0   10:0:SKIPPED (slot 10 arrived during the retry wait)
minute 13  run_due() -> 1   5:2:OK       (the retry keeps its original slot, 5)
history = ["5:1:FAILED", "10:0:SKIPPED", "5:2:OK"];  next_slot -> 15
```

**Example 4: ordering and failures inside one `run_due`**

```
define b "*/10 * *", then a "*/5 * *", then c "*/10 * *"; at minute 10 call run_due()
execution order: a (due since minute 5), b, c (due at 10, in definition order)
a job that fails still counts as one execution and does not stop the others:
two "* * *" jobs, one always throwing, at minute 1: run_due() -> 2
```

**Example 5: first slot of weekday schedules** (epoch minute 0 is Thursday 1970-01-01 00:00 UTC)

```
define "0 0 0"  -> next_slot = 4 320   (Sunday 1970-01-04 00:00)
define "0 12 1" -> next_slot = 6 480   (Monday 1970-01-05 12:00)
define "30 9 *" -> next_slot = 570;  after it ran, next_slot = 1 440 + 570
define "* * *" at minute 0 -> next_slot = 1   (never the minute of definition)
```

## Edge cases

- Invalid definitions are all `JobError::InvalidDefinition`: duplicate name, a cron string without exactly three fields, minute 60, hour 24, weekday 7, `*/0`, a reversed range `5-1`, `max_attempts` 0, an initial or max backoff of 0. Unknown names for `pause`, `resume`, `remove`, `next_slot`, `history` give `JobError::UnknownJob`.
- Exponential growth must not overflow: with a large `k` the delay is simply `max_backoff_ms`.
- A job that fails on its last allowed attempt is `DEAD` (never `FAILED`); with `max_attempts = 1` a single failure is `DEAD`.
- A retry never changes the slot number shown in `history`; the next slot is computed from the current minute when the slot finishes (OK, DEAD) or is skipped.
- While a job is paused, retries wait too; nothing is recorded; `next_slot` is empty.
- After `remove`, `run_due` no longer runs the job, and `history` / `next_slot` for the name are errors.

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

## Running

```bash
cargo test -p p93_cron_job_scheduler
STRICT=1 cargo test -p p93_cron_job_scheduler   # fail instead of skip while unimplemented
cargo test -p p93_cron_job_scheduler --features hidden   # interviewer scenarios
cargo play run 93              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`FakeClock`, `sut!`): `rust/support`.

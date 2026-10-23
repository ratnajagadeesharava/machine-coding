# 106 · Job Scheduler (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 60 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

A massively parallel system runs jobs on a large fleet of machines. Each machine offers a set of **capabilities** (`"gpu"`, `"ssd"`, `"eu-west"`), and every job needs a set of capabilities: it may only run on a machine that offers **all** of them. You are building the scheduler that decides, for each submitted job, which machine gets it. Operators want to choose *how* that decision is made: sometimes "give it to the least busy machine", sometimes "give it to the machine that has finished the most work". The set of ways to decide will grow, and the scheduler is called from many threads at once.

## Problem

Build a job scheduler for the fleet described above.

- Machines are registered with an id and a set of capabilities.
- A job has an id and a set of required capabilities. Submitting it assigns it to exactly one eligible machine, or reports that no machine can run it.
- Which eligible machine wins is decided by a **selection criterion**. Two are required: the machine with the fewest unfinished jobs, and the machine with the most finished jobs. The criterion is chosen per submission, and the scheduler also has a default criterion that callers can change.
- Every machine keeps two counters, unfinished and finished jobs. A job counts as unfinished from the moment it is assigned; when the job is reported finished the machine's unfinished count goes down by one and its finished count goes up by one.
- Whenever several machines are equally good, the lexicographically smallest machine id wins, so the same sequence of calls always gives the same assignments.

## Requirements

1. `add_machine(id, capabilities)` registers a machine with both counters at 0. The capability set may be empty. A blank id, a blank capability, or an id that already exists is an error and changes nothing. The scheduler owns its copy of the capabilities.
2. A machine is **eligible** for a job when its capabilities contain every capability the job requires (exact, case-sensitive string equality). A job that requires nothing is eligible for every machine.
3. `submit_job(job_id, required, criterion)` picks the best eligible machine under `criterion`, assigns the job to it (its unfinished count goes up by one) and returns the machine id. If no machine is eligible it returns `None` and records nothing: the job id remains unused and can be submitted again later.
4. Criterion `FEWEST_UNFINISHED`: the eligible machine with the lowest unfinished count. Criterion `MOST_FINISHED`: the eligible machine with the highest finished count. Only eligible machines compete; the counters of other machines are irrelevant.
5. Ties under either criterion are broken by the lexicographically smallest machine id (plain string comparison; not insertion order, not numeric order of a suffix).
6. `submit_job(job_id, required)` (no criterion) uses the scheduler's default criterion, which is `FEWEST_UNFINISHED` until `set_default_criterion(criterion)` changes it. Passing a criterion to a submission never changes the default.
7. `finish_job(job_id)` moves one job from unfinished to finished on the machine it was assigned to.
8. `unfinished_count(machine)`, `finished_count(machine)` and `machine_of(job)` report the current state.
9. Errors, each leaving the state unchanged: duplicate machine id; unknown machine id in a count query; a job id that is already assigned (finished or not); a blank job id; an unknown criterion name (also when passed to `set_default_criterion`); finishing a job that was never assigned; finishing a job that is already finished.
10. The scheduler is used by many threads at once. Selecting a machine and recording the assignment must behave as one step (two simultaneous submissions never see the same stale counters), a job can be finished by exactly one caller, and the counters must always add up.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- `Result<_, SchedulerError>` with `DuplicateMachine`, `UnknownMachine`, `DuplicateJob`, `UnknownJob`, `AlreadyFinished`, `UnknownCriterion`, `InvalidArgument`; no eligible machine is `Ok(None)`.
- Criterion names are the exact strings `"FEWEST_UNFINISHED"` and `"MOST_FINISHED"` (constants on the contract). Names are case-sensitive.
- `MOST_FINISHED` ignores the current load: a machine with the most finished jobs keeps winning while its unfinished count grows.
- A rejected call changes nothing: not a counter, not the default criterion, not the set of known job ids.
- Fresh scheduler: no machines, default criterion `FEWEST_UNFINISHED`.

## Examples

```
add_machine("m1", {gpu, ssd}); add_machine("m2", {gpu}); add_machine("m3", {ssd})

submit_job("j1", {gpu}, FEWEST_UNFINISHED)        -> m1   (m1, m2 eligible, both 0 unfinished: smallest id)
submit_job("j2", {gpu}, FEWEST_UNFINISHED)        -> m2   (m1 has 1, m2 has 0)
submit_job("j3", {gpu, ssd}, FEWEST_UNFINISHED)   -> m1   (only m1 has both; m1 now has 2 unfinished)
submit_job("j4", {tpu}, FEWEST_UNFINISHED)        -> None  (nobody offers tpu; j4 is not recorded)
submit_job("j4", {}, FEWEST_UNFINISHED)           -> m3   (m3 has 0 unfinished and j4 is a fresh id)
unfinished_count("m1") -> 2     finished_count("m1") -> 0
```

### Worked traces

```
add_machine("a",{x}); add_machine("b",{x}); add_machine("c",{x})

submit_job("j1",{x},MOST_FINISHED)    -> a    (nobody has finished anything: smallest id)
submit_job("j2",{x},FEWEST_UNFINISHED)-> b    (a:1 b:0 c:0 unfinished)
finish_job("j2")                       b: unfinished 0, finished 1
submit_job("j3",{x},MOST_FINISHED)    -> b    (b has the most finished jobs)
submit_job("j4",{x},FEWEST_UNFINISHED)-> c    (a:1 b:1 c:0)
finish_job("j4"); finish_job("j3")      c: finished 1;  b: finished 2
submit_job("j5",{x},MOST_FINISHED)    -> b    (finished: a=0 b=2 c=1)

submit_job("j5",{x},MOST_FINISHED) again       -> error (job id already assigned)
finish_job("j3")                                -> error (already finished)
finish_job("zzz")                               -> error (unknown job)
submit_job("j9",{x},"ROUND_ROBIN")              -> error (unknown criterion); "j9" is still unused
```

```
add_machine("idle",{cpu}); add_machine("busy",{cpu,gpu})
submit_job("g1",{gpu},FEWEST_UNFINISHED) -> busy   submit_job("g2",{gpu},FEWEST_UNFINISHED) -> busy
submit_job("c1",{cpu},FEWEST_UNFINISHED) -> idle   (idle:0 < busy:2; busy's load did not matter to the gpu jobs, only eligibility did)

set_default_criterion("MOST_FINISHED"); submit_job("c2",{cpu}) uses MOST_FINISHED; submit_job("c3",{cpu},"FEWEST_UNFINISHED") uses FEWEST_UNFINISHED
```

### Edge cases

- A job that requires nothing fits every machine, including machines with no capabilities.
- A job needing `{gpu, ssd}` does not fit a machine that has only one of them.
- Capability matching is exact: `gpu` does not match `GPU` or `gpu-a100`.
- Tie-breaking compares ids as strings: `"m10" < "m2" < "m9"`, whatever the insertion order.
- Under `MOST_FINISHED` with nothing finished anywhere, every eligible machine ties and the smallest id wins every time, however loaded it is.
- No machines at all, or none eligible: empty result, no state recorded, job id reusable.
- A job id that is already assigned is rejected even after the job finished.
- A machine added after jobs were submitted competes normally from the next submission on.
- Blank ids and blank capabilities are errors; an empty capability set is fine.
- Failed calls (duplicate machine, duplicate job, unknown criterion, double finish) leave counters, default criterion and known jobs untouched.

## Glossary

- **Machine**: a worker identified by a string id, offering a set of capabilities.
- **Capability**: a label such as `gpu`; a machine offers it or does not.
- **Eligible**: a machine that offers every capability a job requires.
- **Unfinished / finished count**: jobs assigned to a machine that have not / have been reported finished.
- **Selection criterion**: the rule that ranks eligible machines (`FEWEST_UNFINISHED`, `MOST_FINISHED`).
- **Default criterion**: the criterion used by a submission that names none.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
pub const FEWEST_UNFINISHED: &str = "FEWEST_UNFINISHED";
pub const MOST_FINISHED: &str = "MOST_FINISHED";

#[derive(Debug, PartialEq, Eq)]
pub enum SchedulerError {
    DuplicateMachine, UnknownMachine, DuplicateJob, UnknownJob,
    AlreadyFinished, UnknownCriterion, InvalidArgument,
}

pub trait JobScheduler: Send + Sync {
    fn add_machine(&self, machine_id: &str, capabilities: &[&str]) -> Result<(), SchedulerError>;
    fn set_default_criterion(&self, criterion: &str) -> Result<(), SchedulerError>;
    fn submit_job(&self, job_id: &str, required_capabilities: &[&str]) -> Result<Option<String>, SchedulerError>;
    fn submit_job_with(&self, job_id: &str, required_capabilities: &[&str], criterion: &str) -> Result<Option<String>, SchedulerError>;
    fn finish_job(&self, job_id: &str) -> Result<(), SchedulerError>;
    fn unfinished_count(&self, machine_id: &str) -> Result<usize, SchedulerError>;
    fn finished_count(&self, machine_id: &str) -> Result<usize, SchedulerError>;
    fn machine_of(&self, job_id: &str) -> Result<String, SchedulerError>;
}
```

Behaviour of each method is documented in `src/contract.rs`.

Your wiring point (`src/lib.rs`):

```rust
pub fn create() -> Option<Arc<dyn JobScheduler>> { ... }
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. No clock is involved; everything is deterministic. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Eligibility: only machine, no machine, partial capability match, empty requirement, exact matching.
- Both criteria, their tie-breaks, and ranking restricted to eligible machines.
- Counter movement on finish, default criterion, per-call override, machines added later.
- Failure atomicity: unrecorded jobs, duplicate machines/jobs, bad input, unknown criterion, double finish, unknown ids.
- Concurrency: 16 threads submitting 4,000 jobs balance exactly; 8 threads racing to finish the same jobs; mixed submit/finish keeps the counters adding up.
- Harder scenarios (large fleets, interleaved criteria, racing duplicates, machines joining under load) are in `interviewer/`.

## Running

```bash
cargo test -p p106_job_scheduler
STRICT=1 cargo test -p p106_job_scheduler   # fail instead of skip while unimplemented
cargo test -p p106_job_scheduler --features hidden   # interviewer scenarios
cargo play run 106              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`FakeClock`, `sut!`): `rust/support`.

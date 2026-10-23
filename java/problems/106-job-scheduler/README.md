# 106 · Job Scheduler (Java)

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

1. `addMachine(id, capabilities)` registers a machine with both counters at 0. The capability set may be empty. A blank/null id, a null set, a blank/null capability, or an id that already exists is an error and changes nothing. Later changes to the caller's set must not affect the scheduler.
2. A machine is **eligible** for a job when its capabilities contain every capability the job requires (exact, case-sensitive string equality). A job that requires nothing is eligible for every machine.
3. `submitJob(jobId, required, criterion)` picks the best eligible machine under `criterion`, assigns the job to it (its unfinished count goes up by one) and returns the machine id. If no machine is eligible it returns empty and records nothing: the job id remains unused and can be submitted again later.
4. Criterion `FEWEST_UNFINISHED`: the eligible machine with the lowest unfinished count. Criterion `MOST_FINISHED`: the eligible machine with the highest finished count. Only eligible machines compete; the counters of other machines are irrelevant.
5. Ties under either criterion are broken by the lexicographically smallest machine id (plain string comparison; not insertion order, not numeric order of a suffix).
6. `submitJob(jobId, required)` (no criterion) uses the scheduler's default criterion, which is `FEWEST_UNFINISHED` until `setDefaultCriterion(criterion)` changes it. Passing a criterion to a submission never changes the default.
7. `finishJob(jobId)` moves one job from unfinished to finished on the machine it was assigned to.
8. `unfinishedCount(machine)`, `finishedCount(machine)` and `machineOf(job)` report the current state.
9. Errors, each leaving the state unchanged: duplicate machine id; unknown machine id in a count query; a job id that is already assigned (finished or not); a blank job id; an unknown criterion name (also when passed to `setDefaultCriterion`); finishing a job that was never assigned; finishing a job that is already finished.
10. The scheduler is used by many threads at once. Selecting a machine and recording the assignment must behave as one step (two simultaneous submissions never see the same stale counters), a job can be finished by exactly one caller, and the counters must always add up.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `IllegalArgumentException` for blank/null ids, null or invalid capability sets, duplicate machine ids, duplicate job ids, unknown machines, unknown jobs, unknown/null criteria. `IllegalStateException` for finishing a job that is already finished. A submission with no eligible machine returns `Optional.empty()` and is not an error.
- Rust: `Result<_, SchedulerError>` with `DuplicateMachine`, `UnknownMachine`, `DuplicateJob`, `UnknownJob`, `AlreadyFinished`, `UnknownCriterion`, `InvalidArgument`; no eligible machine is `Ok(None)`.
- Criterion names are the exact strings `"FEWEST_UNFINISHED"` and `"MOST_FINISHED"` (constants on the contract). Names are case-sensitive.
- `MOST_FINISHED` ignores the current load: a machine with the most finished jobs keeps winning while its unfinished count grows.
- A rejected call changes nothing: not a counter, not the default criterion, not the set of known job ids.
- Fresh scheduler: no machines, default criterion `FEWEST_UNFINISHED`.

## Examples

```
addMachine("m1", {gpu, ssd}); addMachine("m2", {gpu}); addMachine("m3", {ssd})

submitJob("j1", {gpu}, FEWEST_UNFINISHED)        -> m1   (m1, m2 eligible, both 0 unfinished: smallest id)
submitJob("j2", {gpu}, FEWEST_UNFINISHED)        -> m2   (m1 has 1, m2 has 0)
submitJob("j3", {gpu, ssd}, FEWEST_UNFINISHED)   -> m1   (only m1 has both; m1 now has 2 unfinished)
submitJob("j4", {tpu}, FEWEST_UNFINISHED)        -> empty (nobody offers tpu; j4 is not recorded)
submitJob("j4", {}, FEWEST_UNFINISHED)           -> m3   (m3 has 0 unfinished and j4 is a fresh id)
unfinishedCount("m1") -> 2     finishedCount("m1") -> 0
```

### Worked traces

```
addMachine("a",{x}); addMachine("b",{x}); addMachine("c",{x})

submitJob("j1",{x},MOST_FINISHED)    -> a    (nobody has finished anything: smallest id)
submitJob("j2",{x},FEWEST_UNFINISHED)-> b    (a:1 b:0 c:0 unfinished)
finishJob("j2")                       b: unfinished 0, finished 1
submitJob("j3",{x},MOST_FINISHED)    -> b    (b has the most finished jobs)
submitJob("j4",{x},FEWEST_UNFINISHED)-> c    (a:1 b:1 c:0)
finishJob("j4"); finishJob("j3")      c: finished 1;  b: finished 2
submitJob("j5",{x},MOST_FINISHED)    -> b    (finished: a=0 b=2 c=1)

submitJob("j5",{x},MOST_FINISHED) again       -> error (job id already assigned)
finishJob("j3")                                -> error (already finished)
finishJob("zzz")                               -> error (unknown job)
submitJob("j9",{x},"ROUND_ROBIN")              -> error (unknown criterion); "j9" is still unused
```

```
addMachine("idle",{cpu}); addMachine("busy",{cpu,gpu})
submitJob("g1",{gpu},FEWEST_UNFINISHED) -> busy   submitJob("g2",{gpu},FEWEST_UNFINISHED) -> busy
submitJob("c1",{cpu},FEWEST_UNFINISHED) -> idle   (idle:0 < busy:2; busy's load did not matter to the gpu jobs, only eligibility did)

setDefaultCriterion("MOST_FINISHED"); submitJob("c2",{cpu}) uses MOST_FINISHED; submitJob("c3",{cpu},"FEWEST_UNFINISHED") uses FEWEST_UNFINISHED
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
- Mutating the capability sets you passed in after the call must not change who is eligible.
- Blank/null ids and blank capabilities are errors; an empty capability set is fine.
- Failed calls (duplicate machine, duplicate job, unknown criterion, double finish) leave counters, default criterion and known jobs untouched.

## Glossary

- **Machine**: a worker identified by a string id, offering a set of capabilities.
- **Capability**: a label such as `gpu`; a machine offers it or does not.
- **Eligible**: a machine that offers every capability a job requires.
- **Unfinished / finished count**: jobs assigned to a machine that have not / have been reported finished.
- **Selection criterion**: the rule that ranks eligible machines (`FEWEST_UNFINISHED`, `MOST_FINISHED`).
- **Default criterion**: the criterion used by a submission that names none.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.Optional;
import java.util.Set;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface JobScheduler {
    String FEWEST_UNFINISHED = "FEWEST_UNFINISHED";
    String MOST_FINISHED = "MOST_FINISHED";

    void addMachine(String machineId, Set<String> capabilities);
    void setDefaultCriterion(String criterion);
    Optional<String> submitJob(String jobId, Set<String> requiredCapabilities);
    Optional<String> submitJob(String jobId, Set<String> requiredCapabilities, String criterion);
    void finishJob(String jobId);
    int unfinishedCount(String machineId);
    int finishedCount(String machineId);
    String machineOf(String jobId);
}
```

Behaviour of each method is documented in the Javadoc of `src/main/java/contract/JobScheduler.java`.

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static JobScheduler create()
}
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
mvn test                 # from this directory (or `mvn -pl problems/106-job-scheduler test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`FakeClock`, `Candidate`): `java/testkit`.

# 129 · Job Posting with Candidate Matching (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 60 min | Level 2 — the entry point and the tests are provided; the matching rules, the data model and the concurrency story are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

The core of a hiring site. Recruiters post **jobs** that say which skills are required, which are merely nice to have, how much experience is needed, where the job is and how much it pays at most. Candidates register with their skills, experience, location, the salary they expect and whether they would work remotely. The service answers "how well does this candidate fit this job?" with a number, lists the best candidates of a job and the best jobs of a candidate, and lets eligible candidates apply, withdraw, and recruiters close jobs. Candidates change their profile all the time, and every change must show up in the very next ranking.

## Problem

There are two kinds of records, **jobs** and **candidates**, both identified by a string id chosen by the caller. A candidate is either *eligible* for a job or not (hard rules); an eligible pair has a **match score** (soft preferences). Everything else (rankings, applying) is built on those two ideas. Jobs can be closed; closed jobs stay readable but take no part in matching. The service is used by many threads at once.

## Requirements

### Normalisation and validation

1. **Skills** are compared case-insensitively after trimming: `" Java"` and `"JAVA "` are the same skill. A skill set is a set of distinct normalised skills (duplicates after normalisation collapse into one). Required and nice-to-have sets may be empty. A skill that is in **both** sets of one job counts as **required only** (it is not also a nice-to-have).
2. **Locations** are compared case-insensitively after trimming. Ids are compared exactly (no trimming, case-sensitive).
3. An id, a job title or a location that is null or blank is an error; so is a null skill set or a null/blank skill inside a set; so are a negative `minYears`, `years`, `maxSalary` or `expectedSalary`. Zero is valid for all four. A rejected call changes nothing.
4. `postJob` / `registerCandidate` with an id that already exists is an error (jobs and candidates have separate id spaces).
5. Any operation that names an unknown job or candidate is an error. When both are unknown, the unknown *job* is the one reported.

### Eligibility

A candidate is **eligible** for a job exactly when ALL of these hold:

- the candidate has every required skill of the job;
- `years >= minYears`;
- `expectedSalary <= maxSalary`;
- the candidate's location equals the job's location, **or** the candidate is `remoteOk`.

A closed job has no eligible candidates (see requirement 12).

### Match score

6. `matchScore(jobId, candidateId)` is **0** if the candidate is not eligible or the job is closed. Otherwise it is the sum of five terms:

   | Term | Value |
   |---|---|
   | base | `50` |
   | nice-to-have skills | `10 * min(5, n)` where `n` is the number of the job's nice-to-have skills the candidate has |
   | experience | `min(20, 2 * (years - minYears))` |
   | same location | `10` if the candidate's location equals the job's location (even if `remoteOk` is also set), else `0` |
   | modest salary | `10` if `2 * expectedSalary <= maxSalary`, else `0` |

   An eligible pair therefore scores between 50 and 140; a score of 0 always means "not eligible (or closed)".

### Rankings

7. `topCandidates(jobId, k)` lists the ids of the eligible candidates of an open job, ordered by score descending, then candidate `years` descending, then candidate id ascending (plain string comparison, `compareTo`). It returns at most `k` ids, fewer when fewer candidates are eligible, and an empty list when the job is closed. `k < 1` is an error.
8. `topJobs(candidateId, k)` lists the ids of the open jobs the candidate is eligible for, ordered by score descending, then job id ascending. At most `k` ids. `k < 1` is an error.
9. Rankings always reflect the current state: a candidate who was just registered or updated, or a job that was just posted or closed, is accounted for in the very next call.

### Applications

10. `apply(candidateId, jobId)` records an application. The checks are made in this order, and each failure is an error: unknown ids; the job is closed; the candidate already has an application for this job; the candidate is not eligible (now). A failed call changes nothing.
11. `applications(jobId)` returns the candidate ids of the job's current applications in the order the applications were made, as a new list. `withdraw(candidateId, jobId)` removes the candidate's application (it is an error if there is none); if the candidate applies again later, the new application goes to the **end** of the order. Withdrawing from a closed job is allowed.

### Closing and updating

12. `closeJob(jobId)` closes the job; closing a closed job is an error. A closed job is excluded from `topJobs`, has an empty `topCandidates`, has `matchScore` 0 for every candidate, and accepts no further `apply`. Its existing applications stay readable through `applications` and can still be withdrawn.
13. `updateCandidate(...)` replaces **all** of an existing candidate's data (skills, years, location, expected salary, remote flag). It is effective for every later query at once. Applications the candidate already made stay in place even if the candidate is no longer eligible. An update that fails validation leaves the candidate exactly as it was.

### Constraints

- Many threads post, register, update, rank, apply and close at the same time. The service must be thread-safe. Of several concurrent `apply` calls for the same candidate and job exactly one succeeds; of several concurrent registrations with the same id exactly one succeeds; once `closeJob` has returned, no application can be added to that job; every `apply` that returned normally shows up in `applications`.
- With 5,000 candidates and 500 jobs, 1,000 `topCandidates` calls must finish well within 10 seconds (the hidden scenarios use a tighter bound and concurrent updates).

## Examples

### Example 1: scores and a ranking

```
postJob("backend", "Backend Engineer", required {java, sql}, nice {kafka, redis, docker}, minYears 3, "Pune", maxSalary 100)

candidate  skills                        years  location  expected  remoteOk
alice      Java, SQL, Kafka, Redis        6     Pune        40      no
bob        java, sql, docker, kafka, redis 10   Delhi       60      yes
xena       java, sql, kafka, redis, docker 5    Delhi       40      yes
zed        java, sql, kafka, redis, docker 5    Delhi       40      yes
erin       " JAVA ", " Sql "              20    "pune"      50      no
carol      java                            8    Pune        10      no   (no sql)
dave       java, sql                       9    Mumbai      10      no   (other city, not remote)
frank      java, sql                       2    Pune        10      no   (too little experience)
gina       java, sql                       4    Pune       101      no   (wants more than the job pays)

matchScore(backend, alice) = 50 + 10*2 + min(20, 2*(6-3)=6) + 10 (same city) + 10 (2*40=80 <= 100)          = 96
matchScore(backend, bob)   = 50 + 10*3 + min(20, 2*(10-3)=14) + 0 (Delhi) + 0 (2*60=120 > 100)              = 94
matchScore(backend, xena)  = 50 + 10*3 + min(20, 2*(5-3)=4) + 0 + 10 (80 <= 100)                              = 94
matchScore(backend, zed)   = 94
matchScore(backend, erin)  = 50 + 0 + min(20, 2*(20-3)=34 -> 20) + 10 + 10 (100 <= 100)                       = 90
matchScore(backend, carol/dave/frank/gina) = 0

topCandidates(backend, 10) -> [alice, bob, xena, zed, erin]
      (94 is a tie: bob has more years than xena; xena and zed tie on years, so id order decides)
topCandidates(backend, 3)  -> [alice, bob, xena]
```

### Example 2: the best jobs of a candidate

```
registerCandidate("c1", {java, docker}, years 2, "Pune", expected 40, remoteOk yes)

postJob("A",  .., required {java},       nice {},       minYears 0, "Pune",  100)
postJob("B",  .., required {java},       nice {docker}, minYears 2, "Delhi",  80)
postJob("B2", .., required {java},       nice {docker}, minYears 2, "Delhi",  80)
postJob("D",  .., required {java},       nice {},       minYears 2, "Delhi",  80)
postJob("C",  .., required {java, go},   nice {},       minYears 0, "Pune",  100)

A:  50 + 0  + min(20, 4) + 10 (Pune) + 10 (80 <= 100)  = 74
B:  50 + 10 + 0          + 0         + 10 (80 <= 80)   = 70        B2 = 70
D:  50 + 0  + 0          + 0         + 10              = 60
C:  no "go" -> not eligible

topJobs(c1, 10) -> [A, B, B2, D]           topJobs(c1, 2) -> [A, B]
closeJob("B")   -> topJobs(c1, 10) -> [A, B2, D]      matchScore(B, c1) -> 0
```

### Example 3: applications and updates

```
postJob("J", .., required {java}, nice {}, minYears 2, "Pune", 100)
a: {java}, 2 yrs, Pune, 50, no  -> 50 + 0 + 0 + 10 + 10 = 70
b: {java}, 5 yrs, Pune, 50, no  -> 50 + 0 + 6 + 10 + 10 = 76
c: {go},   5 yrs, Pune, 50, no  -> not eligible

apply(a, J); apply(b, J)         applications(J) -> [a, b]
apply(a, J)  -> error (already applied)       apply(c, J) -> error (not eligible)
withdraw(a, J); apply(a, J)      applications(J) -> [b, a]
updateCandidate(a, {go}, ..)     matchScore(J, a) -> 0     topCandidates(J, 5) -> [b]     applications(J) -> [b, a]
closeJob(J)                      topCandidates(J, 5) -> []   matchScore(J, b) -> 0   applications(J) -> [b, a]
apply(b, J) -> error (closed)    topJobs(b, 5) -> []
```

## Edge cases

- Years equal to `minYears` and salary equal to `maxSalary` are both eligible (the comparisons are `>=` and `<=`). The experience term is then 0.
- The experience term reaches its cap of 20 at 10 years above `minYears`; the nice-to-have term reaches its cap of 50 at five matched skills, and further matches add nothing.
- The salary bonus is judged on `2 * expectedSalary <= maxSalary`: with `maxSalary` 100, an expectation of 50 earns it, 51 does not.
- A job without required skills is open to everybody who meets the other three rules. A candidate without skills can still match such a job.
- A remote candidate in another city is eligible but gets no location bonus; a remote candidate in the job's city gets it.
- A skill listed in both sets of a job counts as required only, so it never adds to the nice-to-have term.
- Different capitalisation or surrounding spaces in skills and locations never create a mismatch; ids are exact, so `"A1"` and `"a1"` are different candidates.
- `k` larger than the number of eligible entries is fine; `k < 1` is an error even if nothing matches.
- A candidate who stops being eligible after applying keeps the application until they withdraw it; they can no longer apply again.
- Scoring and ranking never change anything. Applying and withdrawing change only a job's applications, never the job's or the candidate's data.

## Glossary

- **Eligible**: passes all four hard rules (skills, experience, salary, location/remote).
- **Match score**: 0 for not eligible or closed, otherwise 50 + the four bonus terms of requirement 6.
- **Nice-to-have skill**: a skill that earns points but is not needed to be eligible.
- **Open / closed job**: a closed job takes no part in matching or applying but keeps its applications.
- **Application**: a candidate's recorded interest in one job; at most one per (candidate, job).

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `IllegalArgumentException` for null/blank ids, titles, locations and skills, null skill sets, negative numbers, duplicate ids, unknown ids and `k < 1`. `IllegalStateException` for `apply` on a closed job, a repeated application or an ineligible candidate; for `withdraw` without an application; and for `closeJob` on a closed job.
- `matchScore` returns an `int`; the rankings return `List<String>` of ids; scores are never exposed by the rankings.
- The score, tie-break orders, normalisation and the order of checks in `apply` are exactly as written above.

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.List;
import java.util.Set;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface JobMatching {
    void postJob(String jobId, String title, Set<String> requiredSkills, Set<String> niceToHaveSkills,
            int minYears, String location, long maxSalary);
    void registerCandidate(String candidateId, Set<String> skills, int years, String location,
            long expectedSalary, boolean remoteOk);
    void updateCandidate(String candidateId, Set<String> skills, int years, String location,
            long expectedSalary, boolean remoteOk);
    int matchScore(String jobId, String candidateId);
    List<String> topCandidates(String jobId, int k);
    List<String> topJobs(String candidateId, int k);
    void apply(String candidateId, String jobId);
    void withdraw(String candidateId, String jobId);
    List<String> applications(String jobId);
    void closeJob(String jobId);
}
```

Each method's behaviour is documented in the Javadoc of `src/main/java/contract/JobMatching.java`.

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static JobMatching create()
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. No test sleeps or depends on timing. More scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- The three worked examples above, plus each eligibility rule and its boundary (years, salary, location, remote, missing skill).
- Normalisation of skills and locations, the caps of the nice-to-have and experience terms, the salary bonus boundary, a skill in both sets.
- Ranking order and tie-breaks for both rankings, `k` handling, ineligible and closed entries excluded.
- Duplicate ids, unknown ids and invalid input everywhere, with no side effects after a rejected call.
- Applications: order, duplicates, ineligible, withdraw and re-apply, closed jobs, updates after applying.
- Concurrency: racing identical applications and registrations, parallel applications from many candidates, and a scale run of 5,000 candidates, 500 jobs and 1,000 `topCandidates` calls.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/129-job-candidate-matching test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`Candidate`, `Concurrently`): `java/testkit`.

# Interviewer notes — 93 Cron Job Scheduler

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- First slot strictly after the definition minute, including a definition exactly on a matching minute.
- Weekday and hour/minute combinations that roll to the next day or the next week (e.g. `0 12 1` from a Thursday).
- Range and list parsing: `8-9`, `0,30`, `*/n` for minutes, hours and weekdays; reversed range, `*/0`, out-of-range numbers, wrong field count, empty fields.
- Many missed slots: only the latest runs and the others are not recorded; a 3-week downtime for an every-minute job.
- Backoff cap and overflow with a large `maxAttempts`; a retry due exactly at the boundary millisecond.
- A slot arriving during a retry window is SKIPPED, and the retry still keeps the original slot number.
- Ordering ties by definition order when two jobs have the same due minute; a job that throws a checked exception or an Error-like failure.
- Pause while a retry is pending, resume; remove of a job that is in the due list; 8 threads polling the same tick.

## Likely interviewer questions
- How do you compute the next matching minute? How much work for a rare schedule like `0 0 0`?
- Why does `runDue` run only the latest missed slot? What alternatives (run all, run none) exist and when would a product choose them?
- How do you guarantee that two callers do not run the same attempt? Where do you run user code relative to your lock?
- What exactly is the history entry's slot for a retry, and for a skipped slot?
- What happens to a retry that is pending when the job is paused or removed?

## Invariants
- A job has at most one slot in progress (no overlapping slots).
- An attempt is executed at most once; attempt numbers per slot are consecutive starting at 1.
- The next slot is always strictly in the future of the minute at which it was computed.
- Dead letters are in death order and each failed slot appears exactly once.

## Common mistakes
- Stepping minute by minute to find the next slot (slow after long downtime) or off-by-one so a job runs in the minute of its definition.
- Running all missed slots as a catch-up storm.
- Reading the clock separately for each job in one `runDue`, so ordering and 'due' decisions are inconsistent.
- Holding the global lock while running user code, or running it without claiming the attempt first (double execution).
- Weekday 0 treated as Monday or epoch day 0 treated as a Sunday.
- Backoff `initial << (k-1)` overflowing, or exponent starting from the wrong attempt.
- Letting an exception from one job stop `runDue`.

## Over-engineering traps
- A full Quartz-style cron grammar (seconds, `L`, `W`, `#`, names) when three numeric fields are specified.
- Executor service, worker pools and futures: jobs run on the polling thread by contract.
- A plugin system for retry policies and trigger types before the plain version works.

## Design and scaling discussion
- Polling a list each tick is O(jobs); a min-heap by next due time (plus lazy deletion for pause/remove) gives O(due log n) and lets a real runner sleep until the next deadline.
- Misfire policies (skip, run once, run all) are a product decision; 'run once' is the safest default for idempotent jobs and the one implemented here.
- Exactly-once execution is not achievable in a distributed setup; aim for at-least-once with idempotent jobs, leases to claim attempts and fencing tokens to reject stale workers.
- Persist job definitions and the last completed slot; on restart compute next slot from the last completed one and apply the misfire policy.

## Trade-off questions to ask when the candidate is done
- Run-latest-only versus run-all-missed: effects on idempotent versus non-idempotent jobs.
- Skipping a slot during a retry versus queueing it; how would a user notice?
- Heap of due times versus a scan; when does the scan win?
- How would you design and test the multi-caller claim of an attempt?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.

# Hints — 93 Cron Job Scheduler (Java)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

What is the minimum you need to remember per job to answer 'is something due right now, and if so which attempt of which slot'?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Treat the cron string as three sets of allowed values, and 'next slot' as 'smallest minute > m whose minute, hour and weekday are all in the sets'. Think in terms of days and then minutes inside a day instead of stepping one minute at a time for weeks. Per job, keep a small state: next slot, pending retry (slot, attempt, due time) if any, paused flag.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Per job: parsed fields, `nextSlot`, `retry` (slot, nextAttempt, dueMs), history list. `runDue` collects the jobs that are due (retry due, or `nextSlot <= nowMinute`), sorts them by due time then definition index, and for each claims the attempt atomically (marks it in flight / advances state) before running the user code, so a second caller cannot take it. After the run: success or last failure -> compute the next slot from the current minute; failure with attempts left -> store the retry with `now + min(initial * 2^(k-1), max)` (watch overflow). A due slot while a retry is pending is recorded as skipped.

**Java:** Parse each field into a `boolean[60]`/`boolean[24]`/`boolean[7]` (or `BitSet`); keep jobs in a `LinkedHashMap` to keep definition order, claim state under a lock and run `job.run()` outside it, catching `Throwable` (or at least `Exception`).

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

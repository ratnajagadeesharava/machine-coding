# Follow-ups — 106 Job Scheduler

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — A third criterion
Add `LEAST_RECENTLY_ASSIGNED` (the eligible machine that received a job longest ago, never-assigned machines first, ties by smallest id) and, separately, `FEWEST_CAPABILITIES` (prefer the most specialised machine that still fits). What had to change in your scheduling flow, and what did not? Which of these needs extra state that no existing field holds, and who owns it?

### V3 — Composite and weighted criteria
Callers want "fewest unfinished, ties broken by most finished" and "score = unfinished - 0.5 * finished". How do you express a chain of criteria and a weighted one without a new class per combination? Where do you define the tie-break so it still always ends in the smallest machine id?

### V4 — Capacity, queues and failure
Each machine has a maximum number of concurrent jobs. A submission that fits nowhere right now waits in a queue and is assigned when capacity frees up (on `finishJob`). Machines can also be removed: what happens to their unfinished jobs? Decide what `submitJob` returns for a queued job and what order the queue drains in.

### V5 — Scale
100,000 machines and 10,000 submissions per second. Scanning every machine per submission is too slow. How do you index machines by capability, and keep a ranked structure per criterion up to date as counters change? What is the cost of a submission, of a finish, and of adding a machine, and what do you lock?

### V6 — Priorities, deadlines and retries
Jobs carry a priority and an optional deadline measured on an injectable clock. A job that is not finished in time is retried on a different machine. How do the counters stay correct through a retry, and how do you test it without sleeping?

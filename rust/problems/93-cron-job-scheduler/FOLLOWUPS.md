# Follow-ups — 93 Cron Job Scheduler

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Overlap policy
Add a per-job policy for slots that arrive while a previous run is still executing: SKIP, QUEUE (run after), or ALLOW parallel. Runs now take time and may overlap with other `runDue` callers. What does 'still running' mean and how is it tracked?

### V3 — Time zones and DST
Each job gets a time zone. A 02:30 job on the spring-forward day does not exist, and on the fall-back day it exists twice. Define the semantics, then change your next-slot function.

### V4 — Efficient scheduling
Replace 'scan all jobs on every poll' with a priority queue of due times so a poll costs O(due jobs log n). Pause, resume, remove and reschedule must update the queue without leaking stale entries.

### V5 — Persistence and restart
Persist definitions and run state so that after a crash jobs resume where they were. What do you write before and after running a job to get at-least-once or at-most-once, and how does the misfire rule interact?

### V6 — Distributed workers
Many scheduler nodes poll the same job table. How do you claim an attempt exactly once (lease with expiry, conditional update, fencing token) and what happens when a worker dies in the middle of a run?

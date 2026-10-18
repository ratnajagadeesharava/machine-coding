# Follow-ups — 24 Task Scheduler

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Worker pool
`start()` launches N worker threads that wait for due work instead of the host calling `runDue`. How does a worker sleep until the next due time and wake when something earlier is scheduled?

### V3 — Fixed-delay and cron
Add fixed-delay recurrence (next due = finish time + delay) and a simple cron-like schedule. What differs in your model?

### V4 — Task priorities and dependencies
Tasks with the same due time ordered by priority; a task that only runs after another task succeeded.

### V5 — Persistence and restart
Pending tasks survive a restart. What do you persist, and what happens to occurrences missed while the process was down?

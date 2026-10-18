# Follow-ups — 29 Worker Pool

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Priorities and cancellation
Jobs with priority; a returned handle can cancel a queued job (and optionally interrupt a running one). What happens to the queue slot?

### V3 — Elastic pool
Core and maximum workers, idle timeout. Who decides to start a new worker: the submitter or the pool?

### V4 — Rejection policies
Let the caller choose: reject, run in the caller's thread, block with a timeout, or drop the oldest. How do you keep `submit` simple?

### V5 — Scheduling
`schedule(job, delay)` and fixed-rate jobs on top of the same workers. What data structure holds the delayed jobs?

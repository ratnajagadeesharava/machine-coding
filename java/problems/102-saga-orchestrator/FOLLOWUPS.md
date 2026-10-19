# Follow-ups — 102 Saga Orchestrator

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Timeouts per step
A step can hang. Give each step a deadline and treat expiry as failure. But the step may still complete later: what does that mean for compensation, and how do you make sure the late success is compensated?

### V3 — Parallel branches
Allow a saga to contain steps that run in parallel and then join. Which steps get compensated when one branch fails while another is still running, and in what order?

### V4 — Distributed orchestrator
Run several orchestrator instances against one shared log; any instance can resume a saga whose owner died. How do you detect the dead owner, avoid two instances running the same saga (fencing, see problem 98), and what happens to steps that were in flight?

### V5 — Event-sourced history and queries
Expose the full history of a saga (which step, how many attempts, when) from the log, and add `cancel(sagaId)` that turns a RUNNING saga into compensation. When is cancellation safe?

### V6 — Retry budgets and jitter
Replace the fixed backoff by a per-service retry budget with jitter and a circuit breaker. How does that interact with the exactly-defined sleeps the tests rely on, and where would the policy live?

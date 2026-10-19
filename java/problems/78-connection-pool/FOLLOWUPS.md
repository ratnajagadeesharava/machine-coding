# Follow-ups — 78 Bounded Connection Pool

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Generic pool with a factory
`Pool<T>` creates resources lazily up to `size` via a factory and validates them (`isValid`) before handing out. What if creation is slow or fails?

### V3 — Leak detection / lease time
A holder that keeps a connection for longer than a lease time is reclaimed. How do you stop the old holder from using (or releasing!) a connection that now belongs to somebody else?

### V4 — Fairness
Waiters are served strictly in arrival order. What does that cost in throughput, and how does it interact with timeouts?

### V5 — Graceful shutdown
`closeAndWait(timeout)` waits until all connections are returned, then closes them. What if one never comes back?

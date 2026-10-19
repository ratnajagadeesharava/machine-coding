# Follow-ups — 94 API Gateway

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Weighted and sticky routing
Backends get weights (a with weight 3 serves three of every five requests), and an optional `X-Session` value pins a client to one backend while it is healthy. How does this interact with the per-route counter and with failover when the pinned backend opens its circuit?

### V3 — Retry budget and timeouts
Each request has a deadline. Failover attempts stop when the deadline is gone, and retries across all requests may use at most 10% of traffic. Who owns the budget, and what does the client see when it is exhausted?

### V4 — Distributed rate limiting
Run three gateway instances behind a load balancer; a client's limit must hold across them. Compare a shared store with atomic increments, per-node quotas that are rebalanced, and a sliding-window approximation. What error do you accept?

### V5 — Dynamic configuration
Routes, clients and limits change at runtime without dropping in-flight requests. How do readers see a consistent snapshot? What happens to bucket state and circuit state of a route that is edited?

### V6 — Observability
Expose per-route latency percentiles, per-backend circuit state and per-client rejection counts. What is recorded on the hot path, and what is aggregated off it?

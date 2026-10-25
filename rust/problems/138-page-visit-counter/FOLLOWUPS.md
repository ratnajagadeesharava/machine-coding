# Follow-ups — 138 Multi-Threaded Page Visit Counter

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Approximate unique counting
Pages with tens of millions of distinct visitors make an exact set too expensive. Offer an approximate `uniqueVisitors` with a bounded memory per page (for example a HyperLogLog-style sketch). What error do you promise, how do you test an estimator without flaky tests, and which operations (`visitsBy`, `topVisitors`) can no longer be answered from the sketch?

### V3 — Per-day and per-hour breakdown
`totalVisits(url, day)` and `uniqueVisitors(url, day)` plus a range query over days. An injected clock decides which bucket a visit lands in. How do uniques combine across days (they are not additive), what do you keep and for how long, and does the snapshot guarantee still hold?

### V4 — Bot filtering
Visitors whose visit rate exceeds a threshold, or whose id matches a deny list, must not count. Rules can change at runtime. Do already-recorded visits get retracted when a visitor becomes a bot? What does that do to the "totals never decrease" property of snapshots?

### V5 — Persistence and flush
Counters must survive a restart without writing on every visit. Flush periodically and on shutdown, load on startup. What is lost on a crash and how much do you accept? How do you flush a consistent image without stopping the writers?

### V6 — Sharded counters with periodic aggregation
Visits are written to per-thread or per-shard counters and merged into the published numbers every N milliseconds. What do readers see in between, how stale may a snapshot be, and which of the exact guarantees (total equals the sum of visitors, pair invariants) survive? Where is the trade-off against the strict version?

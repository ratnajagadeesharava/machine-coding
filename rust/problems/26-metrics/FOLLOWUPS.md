# Follow-ups — 26 Metrics Collector

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Gauges and rates
Add gauges (a value that goes up and down and is overwritten) and a `ratePerSecond(name, windowMillis)` derived from the window query.

### V3 — Bounded memory
Window queries must work for a service that runs for months: old events are discarded. What window sizes do you support, and what does a query cost? Bucketed time series vs. an event log.

### V4 — Approximate percentiles
At 100M values per histogram exact answers are too expensive. Switch to a bounded-memory sketch (fixed buckets, t-digest style) with a stated error bound. What does the contract promise now?

### V5 — Exporting
Periodic export of a consistent snapshot to a monitoring system without stopping writers.

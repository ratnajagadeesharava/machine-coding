# Follow-ups — 27 Circuit Breaker

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Failure rate window
Trip on a failure *rate* over the last 60 s (minimum number of calls) instead of consecutive failures. How do you store the window cheaply?

### V3 — Slow calls and timeouts
A call slower than a limit counts as a failure. Who enforces the timeout, and what happens to the abandoned operation?

### V4 — Per-dependency breakers and a registry
Many named breakers with their own settings, a shared metrics view and event hooks for state changes.

### V5 — Bulkhead and fallback
Limit concurrent calls per dependency, and let callers supply a fallback result for refused calls.

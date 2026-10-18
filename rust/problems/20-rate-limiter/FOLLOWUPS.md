# Follow-ups — 20 Rate Limiter

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Per-key limits
Different keys (plans: free, pro, internal) get different limits and windows, changeable at runtime without losing history.

### V3 — Token bucket with burst
Allow a short burst above the steady rate: capacity `B`, refill rate `r` per second. How does the state per key shrink compared with the sliding log?

### V4 — Cheaper memory
Replace the exact log with a sliding window *counter* (two adjacent fixed windows, weighted). What accuracy do you give up, and when is that acceptable?

### V5 — Distributed
The limit must hold across several gateway nodes sharing one store. What operation must be atomic, and what do you do when the store is down — fail open or closed?

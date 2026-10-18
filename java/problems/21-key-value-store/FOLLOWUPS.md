# Follow-ups — 21 In-Memory Key-Value Store

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Persistence
Append-only log plus snapshots, replay on startup. What happens to the transaction stack if the process dies mid-transaction?

### V3 — Active expiry
A background sweeper that removes expired keys so memory does not grow forever. How does it coexist with open transactions?

### V4 — Isolation between clients
Several clients, each with their own transactions: snapshot isolation vs optimistic conflict detection at commit.

### V5 — More types and commands
Counters (`incr`), lists, `expire`/`persist`/`ttl` on existing keys, `watch`. What breaks in your layering?

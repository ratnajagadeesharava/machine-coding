# Follow-ups — 97 Idempotent Request Processor

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Response metadata
Store status and headers with the body and replay them, and let the first caller's original latency be reported separately. What type does your store hold now, and what is the fingerprint of a request?

### V3 — Handler timeouts and crashes
A handler that never returns holds the key forever. Add an in-flight lease: after T seconds another caller may take over. What if the first handler is merely slow and then finishes - how do you stop the double effect?

### V4 — Durable store
Persist keys and responses in a shared database so a restarted or second instance also deduplicates. Which operation must be atomic (insert-if-absent with an in-progress marker), and how do you finish or abandon records left in progress by a crashed instance?

### V5 — Effects that cannot be undone
The handler charges a card and the process dies before the response is stored. How do you make the downstream call itself idempotent (pass the key on), and what do you tell the client on retry?

### V6 — Sharding the key space
Route by key hash to one of N processors so all calls for a key meet at one place. How does it change with a consistent hash ring (problem 96) when a processor leaves?

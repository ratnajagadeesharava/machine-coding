# Follow-ups — 23 Notification System

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Quiet hours and rate limits
Users do not want non-security messages between 22:00 and 07:00 in their timezone, and no more than 5 marketing messages per day. Suppress or delay? Where does the clock come in?

### V3 — Asynchronous delivery with backoff
The gateway call must not block the caller: queue the send, retry with growing delays, give up after a deadline. What does `send` return now, and how does the caller learn the outcome?

### V4 — Digest and batching
Several low-priority messages for one user within a window are combined into one digest message.

### V5 — Delivery log and idempotency
Callers may repeat a request with the same key after a timeout; it must not be delivered twice. Add a per-user history of what was delivered where.

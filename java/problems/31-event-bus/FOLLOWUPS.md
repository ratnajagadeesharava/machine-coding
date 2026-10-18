# Follow-ups — 31 In-Process Event Bus

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Wildcard topics and filters
`orders.*`, `orders.#`, per-subscription predicates. How does the snapshot rule work with patterns?

### V3 — Backpressure and bounded queues
Each subscriber gets a bounded queue with a policy (block publisher, drop oldest, drop newest). What happens to `awaitIdle` and to the ordering promise?

### V4 — Retries and dead letters
A failing callback is retried with backoff; after N failures the event goes to a dead-letter topic.

### V5 — Delivery modes
Synchronous (in the publisher's thread) vs asynchronous per subscription, request/reply, ordered-by-key delivery across many workers.

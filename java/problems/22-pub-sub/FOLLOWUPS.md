# Follow-ups — 22 Pub-Sub Message Broker

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Push delivery
Subscribers register a callback and the broker pushes. What happens when a callback is slow, throws, or re-enters the broker?

### V3 — Bounded queues
Each subscriber has a maximum backlog. Drop oldest, drop newest, or block the publisher? Make it a per-topic setting.

### V4 — Acknowledgements and redelivery
Messages must be acknowledged within a timeout, otherwise they are handed out again. Add consumer groups where each message goes to one member only.

### V5 — Replay
Keep a retained log per topic so a new subscriber can start from the beginning or from an offset. How does that change your storage model?

# Follow-ups — 25 Logging Framework

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Per-appender formatting
Each appender may choose its own layout (plain text, JSON, with/without timestamp). Where does formatting happen, and does it still happen only once per message when two appenders share a layout?

### V3 — Asynchronous appenders
An appender may be slow. Deliver to it from a background queue with a bounded size and a policy (drop oldest, block, drop newest). What do callers see when it is full?

### V4 — Context and structured fields
Attach per-thread context (request id, user) that shows up in every line; allow key/value fields on a message.

### V5 — Rotating file appender
A concrete appender that rolls over by size or by day. How do you test it without a file system?

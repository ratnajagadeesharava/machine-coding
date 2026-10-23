# Follow-ups — 113 Multi-Module Logging Framework

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Retry and dead letters
A failed write is retried up to N times before the caller is told it failed, and logs that still fail go to a dead-letter store that an operator can replay. What happens to ordering and to the "gap-free" promise while a log is being retried? Who is blocked meanwhile?

### V3 — Per-destination buffers
FILE is fast and TOOL is slow, and the slow one is holding everybody up. Give each destination its own buffer and consumer without breaking the global sequence order that `query` exposes. What does "sinks receive lines in increasing order" mean now, and what do you promise to callers?

### V4 — Rotation and retention
Keep only the last N logs (or the last 10 minutes, with an injectable clock) in memory and rotate older ones out. How does `query` behave for a range that was rotated away, and how do you keep the range lookup fast?

### V5 — Dynamic modules and level thresholds
New modules are registered at runtime, and each module has its own minimum level below which logs are dropped (and never consume a number). Where does that decision live, and what is visible to concurrent producers while a threshold changes?

### V6 — Asynchronous acknowledgement
Callers want `log` to return immediately and learn the outcome later (callback or future). What changes about the sequence-number promise, back pressure and shutdown semantics, and what can you still guarantee?

# Follow-ups — 28 Bounded Blocking Buffer

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Fairness and priorities
Waiting producers are served in arrival order; some items can be marked urgent and overtake others. What does this do to your wake-up strategy?

### V3 — Batch operations
`drainTo(maxItems)` and `putAll(list)` that is all-or-nothing within a timeout. How do you keep it atomic and avoid starvation?

### V4 — Shut down without losing work
`closeAndDrain()` returns everything left; `closeNow()` discards. Define the semantics for blocked producers in each case.

### V5 — Lock-free
Single-producer single-consumer ring buffer without locks. What do you give up? What memory ordering is needed?

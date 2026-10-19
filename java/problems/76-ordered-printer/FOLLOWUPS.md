# Follow-ups — 76 Ordered Printer (Turn Taking)

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — FizzBuzz
Instead of a fixed ring, worker `w` may only run when a *predicate on shared state* holds (e.g. 'the next number is divisible by 3 and not by 5'). Generalise the primitive: who decides who is next, and how do you avoid waking everybody?

### V3 — Timeouts
`takeTurn(worker, timeoutMs, action)` gives up when the turn does not come in time. What happens to the ring if the worker whose turn it is never shows up? (skip it? stall? let a supervisor decide?)

### V4 — O(1) wake-ups
Waking all N threads for every hand-over is O(N) work per turn. Wake only the next worker. What primitive gives you that, and what new failure mode appears?

### V5 — Dynamic ring
Workers can join and leave while the ring is running. What if the departing worker is the one whose turn it is?

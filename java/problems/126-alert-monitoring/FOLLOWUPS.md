# Follow-ups — 126 Alert Monitoring System

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Rate-of-change rules
A rule can alert on how fast a metric moves: "the metric grew by more than 50 per minute over the last 5 minutes". Define precisely what is compared (last sample minus first sample in the window, divided by their time distance? least-squares slope?), what happens with fewer than two samples, and what the notification carries as its value. What had to change in the way a rule computes its condition, and what did not (state machine, notifications, retention)?

### V3 — Composite rules
A rule can be `A AND B` or `A OR C` over other rules' conditions (and arbitrary nesting, for example `(A AND B) OR NOT C`). The composite has its own `forMs`, severity and cooldown, and it should not duplicate the sample scans of its parts. What is the value in the notification? Can a composite refer to a rule that is deleted later? How do you prevent cycles?

### V4 — Escalation policies
If a rule stays in `FIRING` for 10 minutes without anyone silencing it, escalate: notify a second, higher-level list of channels (and again after 30 minutes). Escalations are timers measured on the injected clock and driven by `evaluate()`. How do they interact with cooldown, silence and `RESOLVED`? What does `history` show? Where does the "time already escalated" state live?

### V5 — Deduplication and grouping
A hundred rules for one service fire within the same minute. Group alerts by a label (service, region) so a channel receives one grouped message per group per evaluation, with the member list, instead of a hundred. Define the grouping key, the message shape, and what happens when a group has members both firing and resolving in the same evaluation. How does this change the `Notifier` contract?

### V6 — Flapping detection
A rule that changes state more than N times within a period is "flapping": stop notifying every transition, send a single "flapping" notification, and only send another when it has been stable for M minutes. Where is the transition log kept, how much of it, and what bounds its memory? How does it interact with cooldown (is flapping a replacement or an addition)? How would you test it without sleeping?

# Follow-ups — 83 Feature Flag Service

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Typed flags and variants
Flags can return strings or numbers, and an A/B experiment assigns a user to one of N weighted variants. Which parts of your evaluation order survive, and how do you keep the assignment stable when weights change?

### V3 — Richer rules
Rules support `in [..]`, `starts_with`, `>=` on numbers, and AND/OR groups. Keep evaluation fast and keep the first-match semantics. Where do you validate the rule at definition time, and how do you report a bad rule?

### V4 — Audit and history
Every change is recorded with who/when and the previous value, and an admin can roll a flag back to an earlier state. Is the history part of the flag or a separate log? What happens to evaluations that were in flight?

### V5 — Client caching and propagation
SDKs in 10,000 servers poll for changes. How do they learn about a kill switch in under 5 seconds without hammering the service? Compare polling, long polling and push, and decide what a client does when it cannot reach the service.

### V6 — Dependencies and scheduling
A flag can require another flag to be on, and a rollout can ramp from 10% to 100% over a day. How do you evaluate dependent flags without cycles, and how is the schedule driven (injectable clock)?

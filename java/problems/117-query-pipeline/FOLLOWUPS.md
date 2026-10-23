# Follow-ups — 117 Search Query Pipeline

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Per-step timing metrics
For every step, record how many times it ran, how many times it changed the query, how many times it failed and its total / worst duration, readable while traffic is flowing. Where does the measurement live so that no step has to know about it, and how do you test durations deterministically (injectable clock)?

### V3 — Conditional steps
A step can carry a condition: run only when the query has more than three words, only when it contains a given term, or only when an earlier step changed something. Is a skipped-by-condition step in the trace? How does this differ from a disabled step, and who owns the condition?

### V4 — Async and slow steps
Some steps call a remote service (a better spell-checker, a synonym service) that can be slow or down. Give a step a timeout and a fallback: after 50 ms the step counts as failed. What does the caller of `process` get, what happens to the abandoned call, and how do you keep one slow step from exhausting the threads that serve everyone else?

### V5 — Per-request and per-tenant pipelines
Different tenants want different step lists and different dictionaries, and one request can ask to skip a step. How do you share the built-in steps and the configuration between pipelines, how do changes to a shared configuration reach every pipeline, and how do you stop one tenant's registration from affecting another?

### V6 — Step dependencies and early exit
A step can declare that it must run after another step, and a step can answer "the result is final" (for example a cache hit) so the rest are skipped. How do you detect an impossible ordering at registration time, and what appears in the trace for the skipped steps?

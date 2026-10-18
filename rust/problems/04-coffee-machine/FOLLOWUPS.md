# Follow-ups — 04 Coffee Machine

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Drink options
Size (small/large multiplies quantities), extra shots, milk alternatives. Options combine; the recipe table must not explode combinatorially.

### V3 — Outlet capacity and timing
Each outlet prepares one drink at a time and a drink takes time. Users queue for the machine; what is fair? What happens to ingredients reserved for a drink in progress?

### V4 — Maintenance
Descaling is required after N drinks; the machine refuses to serve when it needs it, or when a hopper is empty even if another ingredient could substitute.

### V5 — Low stock alerts
Notify an operator exactly once when an ingredient crosses below its threshold (not once per dispense).

# Follow-ups — 05 Vending Machine

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Card payments
Customers can pay by card for the exact price. Coins and card cannot be mixed in one transaction. What is the shape of "payment" now?

### V3 — Out-of-service and maintenance
Operators open the machine with a key: refilling and cash-out are only possible then, and the customer flow is disabled. Enforce this by construction rather than `if` checks everywhere.

### V4 — Timeouts
If a customer inserts money and walks away for 60 seconds, the machine refunds automatically.

### V5 — Telemetry
Report low stock and low coin inventory to a monitoring service exactly once per crossing, without making the vending logic depend on it.

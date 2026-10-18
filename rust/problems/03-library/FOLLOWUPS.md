# Follow-ups — 03 Library Management

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Policies by membership
Students: 5 loans/21 days; staff: 10 loans/30 days; some books are *reference only* and never leave. New categories should not mean editing every method.

### V3 — Holds expire
A held copy is released if the reserver doesn't collect it within 48 h; the next waiting member is then offered it.

### V4 — Renewals and blocking
A loan can be renewed twice unless someone is waiting. Members with fines over a threshold cannot check out.

### V5 — Notifications and audit
Emit events (due soon, overdue, hold ready) and keep a full audit history per member without coupling the core to delivery.

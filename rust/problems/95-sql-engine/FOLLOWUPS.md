# Follow-ups — 95 Mini SQL Engine

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Transactions
Add `BEGIN`, `COMMIT`, `ROLLBACK` per connection with snapshot reads. Where is the pending write set kept, and what does a concurrent writer to the same row see at commit time?

### V3 — Secondary indexes
`CREATE INDEX ON t(col)` accelerates equality and range filters. How does the planner choose between an index and a scan, and how do UPDATE and DELETE keep the index consistent?

### V4 — More SQL
Add `LEFT JOIN`, `IN (...)`, `LIKE`, `HAVING` and `NULL` with three-valued logic. Which parts of your evaluator and your result typing break first?

### V5 — Durability
Write-ahead log plus periodic snapshots so the database survives a restart. What exactly is logged (statements or row changes), and how is a torn last record detected?

### V6 — Query planning
Make a three-table join cheap: choose the join order and the build side from table sizes. What statistics do you keep, and how do they go stale?

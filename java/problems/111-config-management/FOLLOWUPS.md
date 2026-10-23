# Follow-ups — 111 Configuration Management Service

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Change listeners
`watch(namespace)` lets a caller subscribe to every change (create, update, rollback, delete) in a namespace and be told the key, the old view and the new view. Where are listeners called from, what happens if a listener is slow or throws, and can a listener observe changes out of order?

### V3 — Typed values and schema validation
A namespace can declare a schema (`timeout` is an integer 1..60, `mode` is one of `fast|safe`). Where does validation live, what happens to configs that already exist when a schema is added, and does a rollback have to re-validate an old value?

### V4 — Bounded history and audit
History keeps only the last N versions, and every change records who made it and why. What does `rollback` to a version that was trimmed mean, and how do version numbers stay stable after trimming?

### V5 — Atomic multi-key changes
`updateAll(namespace, changes, expectedVersions)` changes several keys all-or-nothing, and `snapshot(namespace)` returns a consistent view of the whole namespace. How do you avoid deadlock between two overlapping batches, and what does a concurrent `list` see?

### V6 — Persistence and replicas
Configs must survive a restart and be served by three replicas. How do versions double as a replication cursor, what happens on a stale replica read, and how does optimistic concurrency behave across nodes?

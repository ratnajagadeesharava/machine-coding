# Follow-ups — 99 Raft Leader Election

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Log replication
Add `propose(command)` on the leader, an append-entries message with prev-log checks and commit-index advance on majority acknowledgement. Which election rule must change so a stale node cannot win (log up-to-date check)?

### V3 — Pre-vote
A node that is partitioned away keeps incrementing its term and disrupts the cluster when it comes back. Add a pre-vote round that does not bump the term. What exactly is checked before it starts a real election?

### V4 — Persistence
Crash a node and lose its memory: it forgets its vote and can vote twice in a term. Persist term and vote before replying, and model fsync cost. In what order must you persist and send?

### V5 — Membership change
Add a node to a running cluster without ever having two disjoint majorities. Compare single-server changes with joint consensus. What do the quorum calculations look like during the change?

### V6 — Leader leases for reads
Serve linearizable reads at the leader without a round of messages by trusting a time-bounded lease. What clock assumptions does it need, and how does it relate to your check-quorum window?

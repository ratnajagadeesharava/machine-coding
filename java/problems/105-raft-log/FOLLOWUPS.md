# Follow-ups — 105 Raft Log Replication

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Persistence
Today a crash keeps memory. Persist currentTerm, votedFor and the log to a durable store and restart nodes from it. Which writes must complete before replying to a RequestVote or AppendEntries, and what breaks if votedFor is not durable?

### V3 — Snapshots and log compaction
Add `InstallSnapshot` so the log does not grow forever and lagging followers get a snapshot. How do indexes stay consistent after truncation, and what is the follower's prevIndex check now?

### V4 — Linearizable reads
Serve reads from the leader without writing to the log: ReadIndex or leader leases. What does each rely on (heartbeat round, bounded clock drift), and what can go wrong with a deposed leader that does not know yet?

### V5 — Membership changes
Add or remove a node safely (single-server changes or joint consensus). Show why switching the whole configuration at once can elect two leaders in the same term.

### V6 — Pre-vote and leadership transfer
A partitioned node keeps raising its term and disrupts the cluster when it returns. Add the Pre-Vote phase, and a `transferLeadership(target)`. What does each change in the election rules and in the safety argument?

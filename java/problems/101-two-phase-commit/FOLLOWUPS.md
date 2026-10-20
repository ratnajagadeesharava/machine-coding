# Follow-ups — 101 Two-Phase Commit

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Participant timeout before voting
A participant that has not received a decision should be allowed to abort unilaterally only BEFORE it voted YES. Add a rule for how long an unprepared participant remembers a transaction and show that this cannot cause a disagreement.

### V3 — Cooperative termination
If the coordinator is gone, in-doubt participants ask each other: if any participant knows the decision, or voted NO / never prepared, the others can learn or decide. Which cases still block, and why is that unavoidable in 2PC?

### V4 — Three-phase commit
Add a PRE-COMMIT phase so that participants can finish without the coordinator under a no-partition assumption. Show a partition scenario in which 3PC violates atomicity, and explain why production systems use Paxos/Raft-replicated coordinators instead.

### V5 — Batching and group commit
Coordinating thousands of tx/s makes the log fsync the bottleneck. Batch BEGIN and decision records, and discuss what ordering guarantees the single durable append must still give.

### V6 — Read isolation
Today reads ignore in-doubt data. Add read-your-writes snapshot reads, or readers that block on a locked key with a timeout. What does a reader see during the in-doubt window, and what does it cost?

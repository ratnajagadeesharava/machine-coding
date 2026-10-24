# Follow-ups — 124 Kafka-Like Message Streaming Service

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Sticky assignment
Rebalances currently reshuffle almost everything. Change the assignment so that a consumer keeps as many of its previous partitions as possible while the result stays balanced (sizes differ by at most one). Which state do you now need that a pure function of the membership did not need, and what happens to determinism?

### V3 — Rebalance without losing uncommitted work
A consumer has polled messages but not committed when a rebalance moves its partition to someone else. Specify who sees those messages again and how a late commit from the old owner is rejected. How would you introduce a generation/epoch for a membership, and what does a consumer do when it learns its generation is stale?

### V4 — Compaction by key
A topic can be configured so that, per partition, only the latest message for each key is retained (a tombstone message removes the key). Offsets must not be reused or renumbered. What does `earliestOffset`, `messageCount` and polling a compacted range mean now, and when does compaction run relative to publishes?

### V5 — Idempotent producer and consumer lag
Producers attach a producer id and a sequence number so a retried publish is stored exactly once and a gap is rejected. Add a lag metric per group and partition (log end minus committed offset, never negative after retention). Where does the per-producer state live, how is it bounded, and how do you keep both features cheap on the publish hot path?

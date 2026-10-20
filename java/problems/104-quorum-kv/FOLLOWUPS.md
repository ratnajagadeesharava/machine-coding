# Follow-ups — 104 Quorum Key-Value Store

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Tunable consistency
Make W and R per request (ONE, QUORUM, ALL). Which combinations still guarantee that a read sees the latest acknowledged write, and what does a read at ONE show during repair?

### V3 — Hinted handoff and anti-entropy
A replica that was down for hours never sees the 1000 ms re-sends. Add hints stored for it and a background Merkle-tree/digest comparison between replicas. How do you bound the hints' memory?

### V4 — Sharding with a ring
Replace 'every node stores everything' by consistent hashing (problem 96): each key lives on N of M nodes. How do coordinator, quorum and read repair change, and what is a sloppy quorum?

### V5 — Vector clocks instead of timestamps
Replace (time, id) versions by vector clocks so concurrent writes are detected, not silently resolved by the clock. What does `get` return for siblings, and who merges them?

### V6 — Linearizable reads
Even with R = W = majority, a read can return an old value while a write is in progress, and two reads can go backwards. Add a write-back phase before a read replies (ABD) and explain which anomaly it removes and what it costs.

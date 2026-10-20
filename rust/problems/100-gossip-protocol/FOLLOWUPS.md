# Follow-ups — 100 Gossip Protocol with Failure Detection

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Tombstones and deletes
Add `delete(key)`. A delete is a write too: what happens if a stale push re-introduces the deleted key on a node that already forgot it? How long must tombstones be kept and who may garbage-collect them?

### V3 — Delta gossip
Sending the whole state every round is O(data) per message. Send only what the peer is missing (digest exchange, or per-peer version vectors). What does a node need to remember about each peer, and what happens after a restart that loses it?

### V4 — SWIM-style failure detection
Replace 'counter not increased for 2 s' by direct pings with indirect probes through k other nodes and a suspect state before declaring a node dead. What false-positive and detection-time trade-off does that buy compared to pure heartbeat gossip?

### V5 — Dynamic membership
Nodes can join with a seed address and leave gracefully; the peer list is no longer given at construction. How does a new node learn the cluster, how do you tell 'left' from 'dead', and how do you stop a zombie from rejoining with old state?

### V6 — Adaptive fan-out
Make the push rate and fan-out depend on how much news a node has (rumor mongering that stops once the rumor is old) while keeping anti-entropy as the safety net. How do you prove nothing is stuck forever?

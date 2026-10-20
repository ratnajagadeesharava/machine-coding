# Follow-ups — 96 Consistent Hash Ring

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Bounded loads
A hot key range overloads one node. Add a load cap (for example 1.25x the average) that spills a key to the next node on the ring when its owner is full. What state does this add, and what happens to determinism between two processes?

### V3 — Zone-aware replicas
Nodes belong to racks or zones, and `nodesFor` must return replicas in distinct zones when possible. How do you walk the ring, and what do you do when there are fewer zones than replicas?

### V4 — Membership versions
Clients cache the ring and may be one change behind. Introduce a version number and a way to compute which key ranges moved between version v and v+1, so a node can hand them off. What do requests during the handover see?

### V5 — Rendezvous hashing
Replace the ring with highest-random-weight hashing. Compare lookup cost, balance, memory and how replicas fall out. When would you pick it over a ring?

### V6 — Lock-free reads
Make lookups wait-free with an immutable ring snapshot swapped atomically. How expensive is a membership change then, and how do you batch them?

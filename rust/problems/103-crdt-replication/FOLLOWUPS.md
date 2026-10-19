# Follow-ups — 103 CRDT Replicated Store

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Delta state
Send only what changed since the last sync with a given peer (delta-CRDTs). What must a delta carry to stay idempotent, and what do you do when a delta is lost?

### V3 — Causal context for the OR-set
Replace per-element tombstones by a version vector (dotted version vector) so removed state is not kept forever. What does a replica need to know about its peers before it can drop information?

### V4 — Multi-value register
A LWW register silently throws away one of two concurrent writes. Return all concurrent values as 'siblings' using a vector clock, and let the application resolve them. How does the API change?

### V5 — Maps of CRDTs
Add a map from key to a nested CRDT (a map whose values are counters or sets) with remove semantics. What does removing a key mean when a concurrent update to its nested value arrives?

### V6 — Bounded counters and invariants
A bank balance must never go below zero, but a PN-counter cannot enforce that. Sketch a bounded counter that hands out 'rights' to decrement per replica. Which invariants can CRDTs preserve without coordination, and which need consensus?

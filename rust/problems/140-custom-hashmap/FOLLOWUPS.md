# Follow-ups — 140 Custom HashMap

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Survive adversarial keys
An attacker can pick 100,000 keys that all collide in your hasher. Today a lookup then costs time proportional to the number of colliding keys. Change the storage so that the worst case for one bucket is logarithmic, while ordinary buckets stay cheap. What is the threshold at which you switch, and what do you do when the bucket shrinks again?

### V3 — A different collision strategy
Replace how collisions are resolved so that all entries live directly in the table with no per-entry allocation. What happens to `remove`? How does the load factor limit change, and what does a lookup for a missing key cost when the table is nearly full? Keep insertion order and the exact `capacity()` rule.

### V4 — Never pause for a resize
With 50 million entries one `put` currently stalls while everything is moved. Spread the rehash over later operations so that no single call does more than a fixed amount of work. What does `get` do while two tables exist, and how does `capacity()` report during the transition?

### V5 — Scale the concurrency
Operations on different keys should not block each other, and readers should not wait for writers. Where do you split the state, what is the lock order when a resize needs every part, and what happens to `keys()` snapshot atomicity?

### V6 — Bounded memory
Add a maximum number of entries and evict the least recently used one on overflow, as a wrapper around your map. What new state is touched by `get`, and what does that do to your concurrency story?

# Follow-ups — 90 Food Delivery Dispatch

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Partner availability
Partners can go offline and come back (`goOffline`, `goOnline`). An offline partner with an active order finishes it first. What changes in the dispatcher and in your 'free partners' structure?

### V3 — Reassignment on timeout
If a partner does not pick up within 10 minutes (injected clock), the order is taken back and re-dispatched to the next best partner, excluding the one that failed. How do you represent 'excluded' and how do you avoid assigning forever to the same one?

### V4 — Batching
A partner can carry up to two orders from the same restaurant if both are READY. How does the assignment rule become a matching problem and how do you keep the nearest-first guarantee readable?

### V5 — Many partners, many restaurants
With 100k partners a linear scan per order is too slow. Index partners spatially (grid buckets / quadtree) and update on position changes. What do you do about ties and about concurrent reservation of the same partner?

### V6 — Payment and refunds
Orders are charged at PLACED and refunded on cancel, with the restaurant's accept deadline auto-cancelling stale orders. Where do saga steps live and what is idempotent?

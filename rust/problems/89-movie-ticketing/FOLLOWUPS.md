# Follow-ups — 89 Movie Ticket Booking

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Payment timeout and auto-release
Add a `pay` step between hold and booking that can fail or time out. Who decides that a hold is dead if a payment is still in flight when the 5 minutes pass? Design the state machine so a late payment never double-sells a seat.

### V3 — Best available seats
`holdBest(showId, user, count)` picks the best `count` adjacent seats (closest to the middle of the best row). It must still be atomic under contention and never livelock when two users compete for the same block.

### V4 — Dynamic pricing and coupons
Price depends on how full the show is and on time of day, and coupons apply to a booking. Prices are captured at hold time. Which parts of your design change, and what does a refund mean now?

### V5 — Waiting list
When a show is sold out, users can join a waitlist; when a seat is freed by expiry or cancellation it is offered to the first waiting user for 2 minutes. How do you avoid a polling loop, and how does this interact with the hold expiry?

### V6 — Multi-node
The inventory is sharded by show over several nodes and expiry cannot rely on one JVM clock. How do you make 'exactly one buyer per seat' hold with clock skew? Compare TTL in a store with conditional writes versus a single owner per show.

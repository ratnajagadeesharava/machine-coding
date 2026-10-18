# Follow-ups — 10 Flight Booking

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Fare classes and dynamic prices
More than two cabins, fare buckets that sell out one by one (cheapest first), and prices that depend on how full the flight is.

### V3 — Multi-passenger bookings and itineraries
A booking contains several passengers and several flight legs (connections). The whole itinerary is held, paid and cancelled atomically — what happens if the second leg is no longer available?

### V4 — Payment failures and idempotency
Payments are asynchronous and may be retried by the gateway. Confirming twice must not double-charge; a payment that succeeds *after* the hold expired needs a defined outcome (refund? re-validate?).

### V5 — Overbooking and waitlists
Sell up to 105% of capacity; keep a waitlist that is promoted when a seat frees up; compensate bumped passengers.

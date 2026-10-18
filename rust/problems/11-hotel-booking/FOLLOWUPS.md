# Follow-ups — 11 Hotel Booking

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Room preferences and attributes
Guests ask for sea view, accessible rooms, or a specific floor. Matching is no longer by type alone; a booking may be satisfied by an upgrade at no extra cost if nothing else is free.

### V3 — Modifications
Change the dates of an existing booking atomically (keep the old stay if the new one is impossible); extend a stay by a night.

### V4 — Overbooking and waitlists
Sell more than capacity for a type; walk-in guests; a waitlist that is auto-assigned when a cancellation frees a room.

### V5 — Dynamic pricing
Weekend rates, seasons, length-of-stay discounts, rate changes that do not affect existing bookings. A price quote must be reproducible.

# Follow-ups — 13 Bus / Train Seat Booking

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Seat classes and preferences
Window/aisle, sleeper berths, women-only coaches, adjacent seats for groups (book `k` seats together or fail). How do group bookings stay atomic?

### V3 — Waiting list
When full, passengers join a waiting list that is automatically promoted (even partially — a waiting journey may fit after a cancellation of a *different* range).

### V4 — Temporary holds
Seats are held for 5 minutes while the passenger pays. Reuse your design from holds elsewhere (flights, hotels): what changes?

### V5 — Dynamic pricing and partial refunds
Price by distance and occupancy; cancellation fees by time to departure.

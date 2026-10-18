# Interviewer notes — 13 Bus / Train Seat Booking

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- A booking that ends where another starts reuses the same seat.
- A cancelled booking frees only its own segments; neighbours are unaffected.
- `bookSeat` for a seat that is free on the requested segments but booked elsewhere on the trip succeeds.
- Cancel then rebook assigns the lowest free seat again.
- Overlapping-but-not-nested journeys and journeys spanning the whole route.

## Likely interviewer questions
- How do you represent seat occupancy: per seat a bitset over segments, per segment a bitset over seats, interval trees?
- Which representation makes "find the lowest seat free over a range" fast? What is its complexity?
- How do you guarantee atomicity: one lock per trip, per seat, per segment? What is the deadlock risk for the multi-segment variant?
- What exactly is a "seat" and a "booking" in your model?

## Invariants
- For each seat and segment at most one active booking.
- Active booking ⇒ its seat is occupied exactly on its segments.
- `availableSeats(from, to)` equals the number of seats with no occupied segment in the range.
- Refused operations change nothing.

## Common mistakes
- Treating the journey as a single flag per seat (loses the reuse across non-overlapping ranges).
- Off-by-one on segments (marking the destination stop's outgoing segment as occupied).
- Per-seat locks acquired in different orders by different bookings (deadlock) or per-segment locks released too early.
- Computing availability as `seats − bookings overlapping` (wrong when overlaps do not cover every seat).
- Auto-assign picking the first seat *free at the start stop* instead of free throughout.

## Over-engineering traps
- `Coach`, `Berth`, `Route`, `Stop`, `Segment`, `SeatAllocator` interface + strategy, `BookingFactory`. Stop *names* and `int` indexes may be all you need.
- Per-seat objects with listeners.

## Concurrency discussion
- A single `ReentrantLock` per trip is simple, correct, and probably fast enough (the critical section is short). Finer locking only matters if contention is measured.
- Optimistic: bitset per seat updated by CAS over a `long[]`—only for ≤64 segments.

## Trade-offs worth probing
- Bitset per segment (fast "lowest free seat" via AND across segments) vs per seat.
- Coarse lock simplicity vs throughput.

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.

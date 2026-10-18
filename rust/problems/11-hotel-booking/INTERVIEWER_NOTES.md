# Interviewer notes — 11 Hotel Booking

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- Back-to-back stays in the same room (checkout day == next checkin day).
- A stay of exactly one night; a stay spanning many nights; the night `checkOut` itself is *not* occupied.
- Fragmentation: a booking that fits only in a room that already has bookings before/after it.
- Hold expiry at exactly 15 minutes; cancelling an expired hold returns `false`.
- Rooms are assigned lowest-id-first deterministically; ties are not random.
- Type names are case-sensitive and unknown types are refused.

## Likely interviewer questions
- Which concepts do you model: `Room`, `RoomType`, `Booking`, `Hold`, `StayRange`, `Guest`, `Inventory`? Which are values and which have identity?
- How do you represent a room's occupied nights — per-night counters, an interval tree, a sorted list of ranges per room?
- Is there any benefit to separating a `Hold` from a `Booking`, or is it one entity with a status?
- How do you pick a room — first-fit by id, best-fit to reduce fragmentation? What is the impact on revenue?
- How would price rules change over time without altering history?

## Invariants
- For every room and night, at most one active booking or unexpired hold covers it.
- `confirm` never succeeds for an expired hold.
- `total price` of an existing booking never changes after creation.
- A refused request leaves the system unchanged.

## Common mistakes
- Off-by-one on the half-open interval (checkout night counted as occupied).
- Treating availability as "count of bookings < rooms" per type for the *range* instead of room-by-room availability (two bookings can each overlap a different part but together block every room).
- Counting rooms free on night `d` instead of *all* nights in the range when assigning.
- Lazy hold expiry inconsistently applied (some operations see expired holds, others don't).
- Check-then-act race: two threads see the same free room.

## Over-engineering traps
- `HotelManager`, `BookingService`, `RoomService`, `PricingStrategy`, `HoldTimerService` interfaces from the start.
- A scheduler thread to expire holds when comparing timestamps lazily is simpler and race-free.
- `Room` subclasses per type.

## Scalability / concurrency discussion
- Per-room locks acquired in a global order, or a lock per (type, night range) bucket; optimistic retry.
- Interval storage: sorted array/tree per room; per-type segment tree of free-room counts.
- Moving to a database: exclusion constraints / `SELECT … FOR UPDATE` per room, hold rows with `expires_at`.

## Trade-offs worth probing
- First-fit vs best-fit allocation; availability counts vs room-level search.
- Locking per room (finer, needs a multi-room scan strategy) vs per type.

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.

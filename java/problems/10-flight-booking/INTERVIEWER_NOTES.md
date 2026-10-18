# Interviewer notes — 10 Flight Booking

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- The hold boundary: valid at 9 min 59.999 s, expired at exactly 10 min.
- Exactly 24 h before departure → full refund; 1 ms later → 50%.
- 50% refund rounds down for odd fares.
- A failed payment does **not** extend the hold and does **not** release it.
- Cancelling an expired reservation, or a booking twice, is refused.
- The same passenger can hold several seats; the same seat can be re-reserved after expiry by someone else.
- A flight with every seat held disappears from `search` and reappears when holds expire.

## Likely interviewer questions
- Which concepts do you model — `Flight`, `Seat`, `Reservation`, `Booking`, `Passenger`, `Fare`? Which of those are just data, which have behavior? Is a `Booking` different from a `Reservation`, or is it a state?
- Where does hold expiry live? A timer thread, a lazy check at read time, a scheduler? Trade-offs?
- What is the unit of locking — per flight, per seat, global? What operation needs two seats at once (V3)?
- How would the refund policy change per fare class without touching `cancel`?
- How do you keep `search` fast when seats change constantly?

## Invariants
- A seat is in exactly one state: free, held (with expiry), booked.
- A reservation id maps to exactly one seat and one passenger; a booking keeps the id.
- `free + held + booked = capacity` per flight.
- Refund ≤ fare and is paid at most once.

## Common mistakes
- Lazy expiry implemented only in `availableSeats`, so `reserve`/`confirm`/`search` disagree about whether a hold is alive.
- Check-then-act on seat state (`isAvailable()` then `reserve()`) with no atomicity.
- Using a background thread to expire holds and then racing with `confirm`.
- Confirming an expired reservation because the cleanup hasn't run yet.
- Locking the whole system for every operation (correct but kills throughput) — or locking per seat but not handling the multi-seat case.
- Storing `Instant`/wall-clock reads inside domain objects instead of using the injected clock.

## Over-engineering traps
- Class hierarchy `EconomySeat extends Seat`, `BusinessSeat extends Seat` for what is a cabin attribute.
- `BookingService`, `PaymentService`, `NotificationService`, `SeatLockProvider` interfaces and factories in V1.
- A full state-machine framework for `Reservation` with a transition table when three states are enough.
- Separate microservice-style repositories for 5 entities.

## Scalability / concurrency discussion
- Per-flight lock or per-seat CAS; lazy expiry (timestamp compared at access) removes the need for a sweeper.
- Searching: index flights by (origin, destination) sorted by departure; maintain an available-seat counter per flight updated atomically.
- Hot flights: sharding, optimistic concurrency, and how the user experience of a lost race should look.

## Trade-offs worth probing
- Lazy vs eager expiry (clock-driven correctness vs timers).
- One lock per flight (simple, enough?) vs one per seat (fine-grained).
- Reservation→Booking as one entity with status vs two entities.

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.

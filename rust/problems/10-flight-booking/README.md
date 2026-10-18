# 10 · Flight Booking (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Build the booking core of an airline reservation system: customers search flights, hold a seat while they pay,
confirm the booking, or cancel and receive a refund according to the fare rules.

## Requirements

1. Flights connect an origin to a destination at a departure time. Each flight has **business** and **economy** seats; every seat has a label (`B1…Bn`, `E1…En`) and each cabin has its own fare.
2. **Search** by origin and destination for flights departing at or after a given time. Results are ordered by departure time, then cheapest economy fare, then flight id. Flights with no seat currently available are not returned.
3. A customer **reserves** a specific seat for a passenger. A reservation **holds** the seat for **10 minutes**; while held (and not expired) nobody else can reserve it.
4. A reservation is **confirmed** by a payment attempt. A *failed* payment keeps the hold (the customer may retry until the hold expires); a successful one turns the reservation into a confirmed booking that no longer expires.
5. An expired hold releases the seat automatically — no cleanup call is made by anyone — and can no longer be confirmed.
6. A seat can never be held or booked by two customers at the same time, including under heavy concurrency.
7. **Cancellation**: cancelling a confirmed booking at least **24 hours** before departure refunds the full fare; later than that, **50%** (rounded down). Cancelling an unexpired, unconfirmed reservation releases the seat with a refund of 0. A seat released by cancellation is available again.
8. Flights that have already departed cannot be reserved.

## Constraints

- Thousands of flights, hundreds of seats each; searching must not scan every flight linearly for every query in a production setting (a scan is fine for the tests, be ready to discuss an index).
- Time comes from the `Clock` given to the factory.
- Operations must be safe to call from many threads.

## Examples

```
addFlight("F1", "BLR", "DEL", dep, economy=3, business=1, economyFare=5000, businessFare=20000)
reserve("F1", "E2", "Asha")     → reservation R1     (E2 is held for 10 minutes)
reserve("F1", "E2", "Ravi")     → refused
confirm(R1, paymentSucceeds=false) → false, E2 still held
confirm(R1, true)               → true               (booking, no expiry)
cancel(R1) 48h before departure → refund 5000        cancel(R1) again → refused
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- Seat labels are exactly `B1…B<business>` and `E1…E<economy>`. `availableSeats` returns every seat that is neither booked nor under an unexpired hold: business seats first, then economy, each in numeric order. Unknown flight → empty list.
- `reserve` returns a reservation id, or empty if: the flight or seat is unknown, the seat is taken, the flight has already departed, or the passenger name is blank. A hold is valid while `now < reservedAt + 10 min` (at exactly 10 minutes it has expired).
- `confirm(id, paymentSucceeds)` returns `true` only when the reservation is active (unexpired, unconfirmed) **and** payment succeeded; in every other case `false`. A confirmed booking keeps the **same id** as the reservation.
- `cancel(id)` returns the refund amount (possibly 0) or empty when the id is unknown, expired, or already cancelled. Refund: `fare` if `departure − now ≥ 24 h`, else `fare / 2` (integer division).
- `search` only returns flights that still have at least one available seat.
- `fareOf(flight, seat)` is the fare of that seat's cabin; empty for unknown flights/seats.

## Test entry point (Rust)

The tests drive your system through the thin, primitive-typed contract below (`src/contract.rs`). It is
**not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up
in `src/lib.rs`.

```rust
//! Thin test entry point. Your domain model lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

pub trait FlightBookingApi: Send + Sync {
    #[allow(clippy::too_many_arguments)]
    fn add_flight(
        &self,
        flight_id: &str,
        from: &str,
        to: &str,
        departure_millis: u64,
        economy_seats: u32,
        business_seats: u32,
        economy_fare: u64,
        business_fare: u64,
    );

    /// Flights with at least one available seat, ordered by departure, economy fare, then id.
    fn search(&self, from: &str, to: &str, not_before_millis: u64) -> Vec<String>;

    /// Business seats first, then economy; each in numeric order.
    fn available_seats(&self, flight_id: &str) -> Vec<String>;

    /// Holds the seat for 10 minutes and returns the reservation id, or `None` if refused.
    fn reserve(&self, flight_id: &str, seat: &str, passenger: &str) -> Option<String>;

    /// True only if the reservation is active and payment succeeded.
    fn confirm(&self, reservation_id: &str, payment_succeeds: bool) -> bool;

    /// Refund amount (possibly 0), or `None` if unknown/expired/already cancelled.
    fn cancel(&self, reservation_or_booking_id: &str) -> Option<u64>;

    fn fare_of(&self, flight_id: &str, seat: &str) -> Option<u64>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(_clock: Arc<dyn Clock>) -> Option<Arc<dyn FlightBookingApi>> {
    None
}
```

## Language notes

The trait is `&self` + `Send + Sync`. Think about where the lock lives (per flight? per seat?) and whether expiry is checked lazily using `Clock::now_millis`.

## Running

```bash
cargo test -p p10_flight_booking
STRICT=1 cargo test -p p10_flight_booking   # fail instead of skip while unimplemented
cargo test -p p10_flight_booking --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p10_flight_booking --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

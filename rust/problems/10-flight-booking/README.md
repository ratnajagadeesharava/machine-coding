# 10 · Flight Booking (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building the booking core of an airline reservation system. Customers search for flights between two cities,
pick a specific seat, and get a **10-minute hold** on it while they pay. If payment succeeds the booking is
confirmed for good; if the hold runs out first the seat quietly goes back on sale. A customer can also cancel and get
money back, with the amount depending on how close to departure they cancel. Many customers use the system at the
same time, so two people must never end up with the same seat.

### Terms

- **Flight** - identified by an id, with an origin, a destination and a departure time (epoch milliseconds).
- **Cabin / fare** - each flight has a **business** and an **economy** cabin; every seat in a cabin costs that cabin's fare (a whole number of currency units).
- **Seat label** - `B1..B<business>` for business, `E1..E<economy>` for economy (1-based).
- **Reservation (hold)** - a temporary, unpaid claim on one seat. Its id is returned by `reserve`.
- **Booking** - a reservation after a successful payment. It keeps the **same id** as the reservation and never expires.
- **Available seat** - a seat that is neither booked nor under an unexpired hold.
- **Clock** - time always comes from the `Clock` passed to the factory (tests move it by hand), never from the system clock.

## Requirements

1. Flights connect an origin to a destination at a departure time. Each flight has **business** and **economy** seats; every seat has a label (`B1…Bn`, `E1…En`) and each cabin has its own fare.
2. **Search** by origin and destination for flights departing at or after a given time (`departure >= not_before`). Results are ordered by departure time, then cheapest economy fare, then flight id. Flights with no seat currently available are not returned; a flight that is full only because of unexpired holds reappears once those holds expire.
3. A customer **reserves** a specific seat for a passenger. A reservation **holds** the seat for **10 minutes**; while held (and not expired) nobody else can reserve it.
4. A reservation is **confirmed** by a payment attempt. A *failed* payment keeps the hold (the customer may retry until the hold expires); a successful one turns the reservation into a confirmed booking that no longer expires. Confirming an already confirmed booking returns `false`.
5. An expired hold releases the seat automatically - no cleanup call is made by anyone - and can no longer be confirmed or cancelled.
6. A seat can never be held or booked by two customers at the same time, including under heavy concurrency.
7. **Cancellation**: cancelling a confirmed booking at least **24 hours** before departure refunds the full fare of the seat's cabin; later than that, **50%** (rounded down). Cancelling an unexpired, unconfirmed reservation releases the seat with a refund of 0. A seat released by cancellation is available again.
8. Flights that have already departed cannot be reserved.

## Constraints

- Thousands of flights, hundreds of seats each; searching must not scan every flight linearly for every query in a production setting (a scan is fine for the tests, be ready to discuss an index).
- Time comes from the `Clock` given to the factory.
- Operations must be safe to call from many threads.

## Examples

Flight `F1` BLR to DEL departing in 72 h, 3 economy seats (fare 5000), 1 business seat (fare 20000). `now` is T.

```
available_seats("F1")                -> [B1, E1, E2, E3]
reserve("F1","E2","Asha")           -> R1               available_seats = [B1, E1, E3]
reserve("F1","E2","Ravi")           -> empty            (held by R1)
reserve("F1","E9","Ravi")           -> empty            (no such seat)
confirm(R1, payment_succeeds=false)  -> false            E2 is still held
(T + 5 min) confirm(R1, true)       -> true             R1 is now a booking, never expires
confirm(R1, true)                   -> false            (already confirmed)
cancel(R1) at 48 h before departure -> 5000             E2 available again
cancel(R1)                          -> empty            (already cancelled)
```

Hold expiry: `reserve("F1","E2","Asha")` at T. At `T + 10 min - 1 ms` the hold is still valid and `confirm(id, true)` is
`true`. At exactly `T + 10 min` it has expired: E2 is in `available_seats` again, `confirm(id, true)` is `false`,
`cancel(id)` is empty, and someone else can reserve E2.

Refund timing for a confirmed business seat (fare 20000) departing at D:

| Cancelled at | Time before departure | Refund |
|---|---|---|
| D - 48 h | 48 h | `20000` |
| D - 24 h | 24 h exactly | `20000` |
| D - 23 h | 23 h | `10000` |

Search: flights F0 (departs 5 h before F1, fare 9000), F1 (fare 5000), F2 (same departure as F1, fare 4000), all
BLR to DEL: `search("BLR","DEL",T)` -> `[F0, F2, F1]` (earlier departure first; same departure, cheaper economy fare first).
`search("BLR","DEL", departure_of_f1)` -> `[F2, F1]`. `search("DEL","BLR",T)` -> `[]`.

## Edge cases

- `reserve` with a blank passenger name (`" "`), an unknown flight, or an unknown seat returns empty. A seat that does not exist for the flight (`E4` on a 3-economy-seat flight) also has no fare.
- A flight with `0` business seats simply has no `B` labels.
- Business seats are listed before economy seats in `available_seats`; within a cabin the order is numeric (`E2` before `E10`).
- Reserving after the flight's departure time has passed returns empty.
- A failed payment never changes the hold's expiry time: the 10 minutes still run from the original `reserve`.
- Many threads racing for one seat: exactly one `reserve` succeeds. Many threads racing across many seats: no seat is ever handed out twice.

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- Seat labels are exactly `B1…B<business>` and `E1…E<economy>`. `available_seats` returns every seat that is neither booked nor under an unexpired hold: business seats first, then economy, each in numeric order. Unknown flight → empty list.
- `reserve` returns a reservation id, or empty if: the flight or seat is unknown, the seat is taken, the flight has already departed, or the passenger name is blank. A hold is valid while `now < reservedAt + 10 min` (at exactly 10 minutes it has expired).
- `confirm(id, payment_succeeds)` returns `true` only when the reservation is active (unexpired, unconfirmed) **and** payment succeeded; in every other case `false`. A confirmed booking keeps the **same id** as the reservation.
- `cancel(id)` returns the refund amount (possibly 0) or empty when the id is unknown, expired, or already cancelled. Refund: `fare` if `departure − now ≥ 24 h`, else `fare / 2` (integer division).
- `search` only returns flights that still have at least one available seat.
- `fare_of(flight, seat)` is the fare of that seat's cabin; empty for unknown flights/seats.

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

The trait takes `&self` and is `Send + Sync`: the concurrency tests call it from many threads at once. Time is read from the `Clock` given to the factory (`Clock::now_millis`). Ids, times and money are unsigned integers (`u64`); seat counts are `u32`.

## Running

```bash
cargo test -p p10_flight_booking
STRICT=1 cargo test -p p10_flight_booking   # fail instead of skip while unimplemented
cargo test -p p10_flight_booking --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p10_flight_booking --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

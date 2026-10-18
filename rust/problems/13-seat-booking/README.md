# 13 · Bus / Train Seat Booking (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 90 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

An intercity bus or train runs through an ordered list of stops. Passengers buy a seat from one stop to a later stop.
A seat that is taken from A to C is free again for someone travelling from C to D — the same physical seat can be
sold several times along the route. Build the booking engine.

## Requirements

1. A **trip** has an id, an ordered list of stops and a number of seats numbered `1..N`.
2. A **booking** is for a passenger from a boarding stop to a destination stop that comes **later** in the route. It occupies its seat on every segment between consecutive stops in `[from, to)`.
3. `book` automatically assigns the **lowest-numbered seat** that is free for the whole journey. `bookSeat` requests a specific seat.
4. Two bookings may share a seat only if their journeys do not overlap (one may end where the next one starts).
5. A booking can be **cancelled**, which frees its seat for exactly the segments it occupied.
6. Availability: how many seats are free for the whole journey between two stops.
7. Invalid requests (unknown trip or stop, `from` not before `to`, seat out of range) are refused without side effects.
8. The same seat must never be sold twice for an overlapping journey, even when many passengers book concurrently.

## Constraints

- Up to 200 stops and 500 seats per trip; many trips.
- Thread-safe. Booking contention on a popular trip must not allow double-selling.
- All checks and the assignment of a seat must be atomic for a booking.

## Examples

```
trip T: stops [A,B,C,D], 2 seats
book(T, A, C, "p1") → seat 1      book(T, C, D, "p2") → seat 1      (no overlap)
book(T, B, D, "p3") → seat 2      (seat 1 is busy on B–C)           availableSeats(T, B, C) = 0
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `book`/`bookSeat` return a booking id, or empty if refused. `seatOf(id)` is the assigned seat; empty for unknown or cancelled bookings.
- `availableSeats(trip, from, to)` = seats free on **every** segment of the journey; 0 for invalid journeys.
- `cancel(id)` returns `true` the first time, `false` for unknown or already cancelled ids.
- Stops are identified by name; `from` must appear strictly before `to` in the trip's stop list.

## Test entry point (Rust)

The tests drive your system through the thin, primitive-typed contract below (`src/contract.rs`). It is
**not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up
in `src/lib.rs`.

```rust
//! Thin test entry point. Your domain model lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

pub trait SeatBookingApi: Send + Sync {
    fn add_trip(&self, trip_id: &str, stops: &[&str], seats: u32);

    /// Lowest-numbered seat free for the whole journey. Returns the booking id, or `None` if refused.
    fn book(&self, trip_id: &str, from: &str, to: &str, passenger: &str) -> Option<String>;

    /// A specific seat (1..=N).
    fn book_seat(
        &self,
        trip_id: &str,
        seat: i32,
        from: &str,
        to: &str,
        passenger: &str,
    ) -> Option<String>;

    fn seat_of(&self, booking_id: &str) -> Option<u32>;

    fn cancel(&self, booking_id: &str) -> bool;

    /// Seats free on every segment of the journey; 0 for invalid journeys.
    fn available_seats(&self, trip_id: &str, from: &str, to: &str) -> u32;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create() -> Option<Arc<dyn SeatBookingApi>> {
    None
}
```

## Language notes

The trait is `&self` + `Send + Sync`. A single `Mutex` per trip inside a map guarded by `RwLock` is a very reasonable first design; justify it.

## Running

```bash
cargo test -p p13_seat_booking
STRICT=1 cargo test -p p13_seat_booking   # fail instead of skip while unimplemented
cargo test -p p13_seat_booking --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p13_seat_booking --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

# 11 · Hotel Booking (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Design the reservation engine of a hotel: guests book a room *type* for a range of nights, the system assigns a
concrete room, supports temporary holds while the guest pays, and guarantees a room is never promised twice for the same night.

## Requirements

1. The hotel has **rooms**, each with an id, a **type** (e.g. `SINGLE`, `DOUBLE`, `SUITE`) and a nightly rate. Guests book a **type**, not a specific room.
2. A stay covers nights `[checkIn, checkOut)` (days are integers): checking out on day `d` frees the room for a guest checking in on day `d`.
3. `book` confirms a stay immediately. The system assigns the free room of that type with the **lexicographically smallest id** that is free for **all** the requested nights.
4. `hold` reserves a room for the same kind of request but only for **15 minutes**; `confirm` turns an unexpired hold into a booking. Expired holds release the room automatically (nobody calls cleanup).
5. `cancel` releases a booking or a hold.
6. Availability queries: how many rooms of a type are free on a given night (neither booked nor held).
7. The price of a stay is `nights × nightly rate` of the assigned room.
8. Requests with `checkOut <= checkIn`, an unknown type, or no room free for the whole range are refused.
9. No room may ever be booked or held twice for an overlapping night — even under concurrent requests.

## Constraints

- Hundreds of rooms, stays up to a year, tens of thousands of bookings. Availability must not require scanning every booking when asked about a single night (be ready to discuss a better structure).
- Time comes from the `Clock` given to the factory.
- Operations must be safe to call from many threads.

## Examples

```
rooms: R1, R2 (DOUBLE, 100/night)
book(alice, DOUBLE, 1, 4) → B1 in R1          availableRooms(DOUBLE, 2) = 1
book(bob,   DOUBLE, 4, 6) → B2 in R1          (checkout day 4 == checkin day 4)
book(carol, DOUBLE, 2, 5) → B3 in R2
book(dave,  DOUBLE, 3, 4) → refused           (R1 busy, R2 busy)
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `book`/`hold` return an id, or empty when refused. The id of a hold stays the id of the booking after `confirm`.
- `roomOf(id)` is the assigned room id; empty for unknown/cancelled ids. `totalPrice(id)` is `nights × rate`.
- A hold is valid while `now < heldAt + 15 min`. `confirm` returns `true` only for an active (unexpired, unconfirmed) hold.
- `cancel(id)` returns `true` if a booking or an unexpired hold was released; `false` for unknown, expired, or already-cancelled ids.
- `availableRooms(type, day)` counts rooms of that type with no booking and no unexpired hold covering **night** `day`.

## Test entry point (Rust)

The tests drive your system through the thin, primitive-typed contract below (`src/contract.rs`). It is
**not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up
in `src/lib.rs`.

```rust
//! Thin test entry point. Your domain model lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

pub trait HotelBookingApi: Send + Sync {
    fn add_room(&self, room_id: &str, room_type: &str, nightly_rate: u64);

    /// Confirmed stay for nights [check_in_day, check_out_day). Returns the booking id, or `None` if refused.
    fn book(
        &self,
        guest: &str,
        room_type: &str,
        check_in_day: i32,
        check_out_day: i32,
    ) -> Option<String>;

    /// Same as `book`, but the room is only held for 15 minutes.
    fn hold(
        &self,
        guest: &str,
        room_type: &str,
        check_in_day: i32,
        check_out_day: i32,
    ) -> Option<String>;

    /// True only for an active (unexpired, unconfirmed) hold.
    fn confirm(&self, hold_id: &str) -> bool;

    /// Releases a booking or an unexpired hold.
    fn cancel(&self, id: &str) -> bool;

    /// Rooms of the type free (no booking, no unexpired hold) on the given night.
    fn available_rooms(&self, room_type: &str, day: i32) -> u32;

    fn room_of(&self, id: &str) -> Option<String>;

    fn total_price(&self, id: &str) -> Option<u64>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(_clock: Arc<dyn Clock>) -> Option<Arc<dyn HotelBookingApi>> {
    None
}
```

## Language notes

The trait is `&self` + `Send + Sync`. Days are `i32`; the clock gives milliseconds.

## Running

```bash
cargo test -p p11_hotel_booking
STRICT=1 cargo test -p p11_hotel_booking   # fail instead of skip while unimplemented
cargo test -p p11_hotel_booking --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p11_hotel_booking --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

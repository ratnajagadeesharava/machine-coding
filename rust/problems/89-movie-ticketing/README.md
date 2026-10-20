# 89 · Movie Ticket Booking (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 75 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Design the booking engine of a movie-ticketing app. Theatres have screens with seat grids, screens run shows, and
customers search for a movie in their city, pick seats and pay. Because payment takes a while, picking seats must reserve them
for a short time so nobody else can buy them in the meantime; if the customer does not complete in time the seats must become
available again by themselves. Premium rows cost more than regular rows. Customers can cancel a booking and get a refund that
depends on how close the show is. On a Friday night thousands of people fight for the same seats.

## Requirements

1. `addScreen(theatre, city, screen, rows, seatsPerRow)`: rows 1..26 are labelled A, B, C ...; seats are 1-based, so labels look like `A1`, `C10`. Rows A and B are PREMIUM, all other rows REGULAR. An already existing screen is refused.
2. `addShow(showId, theatre, screen, movie, startMinute, durationMinutes, regularPrice, premiumPrice)`. Unknown screen, non-positive duration or price, or a duplicate show id are invalid input; a show that overlaps another show on the same screen is a state conflict (touching shows are fine).
3. `searchShows(city, movie)` returns the ids of shows of that movie in that city that have not started yet, sorted by start then id.
4. `availableSeats(showId)` returns the seats that can be held right now (not sold, not under a live hold) in row-then-number order.
5. `hold(showId, user, seats)` reserves all the seats for the user for 5 minutes, all-or-nothing, and returns a hold id. Unknown show/seat, an empty list or duplicates are invalid; a seat that is sold or held, or a show that has already started, is a conflict.
6. `release(holdId, user)` gives a hold up early. `confirm(holdId, user)` turns a live hold into a booking and returns the booking id.
7. `seatsOf(bookingId)` returns the sorted labels; `totalOf(bookingId)` is the sum of the seat prices (premium rows at the premium price, others at the regular price).
8. `cancel(bookingId, user)` frees the seats and returns the refund: 100% of the total if the show starts in 120 minutes or more, 50% (rounded down) if 30..119 minutes away, nothing if closer or already started.

## Constraints

- The system is used from many threads. A seat must never be sold or held twice: if 16 threads hold the same seat at the same instant, exactly one succeeds.
- Holding several seats is atomic: a failed hold (because one seat is taken) must not leave any of the other seats held, even briefly visible as unavailable forever.
- Time comes from an injected clock (`Solution.create(clock)`); "now" in minutes is `clock.millis() / 60_000`. Never sleep; expired holds must free their seats as soon as the clock passes the expiry, without a background thread being required.
- Search must stay quick with 2,000 shows in the system (hidden test); `availableSeats` must reflect expired holds exactly at the 5-minute mark.
- Prices and refunds are whole numbers (`long` / `u64`); integer division rounds down.

## Examples

```
screen s1: 4 rows x 5 seats (A,B premium), show1 starts at minute 600, regular 200, premium 350
hold(show1, ann, [B2, A1, C3])     -> holdId       (available: 17 seats left)
  4 minutes later: still held;  5 minutes later: seats free again, confirm(holdId) -> IllegalStateException
confirm(holdId, ann)  -> bookingId;  seatsOf -> [A1, B2, C3];  totalOf -> 350+350+200 = 900
cancel(bookingId, ann): 100% (600 min away), 50% (100 min away), 0 (25 min away)
```

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `TicketingApi` with `IllegalArgumentException` (invalid input), `IllegalStateException` (conflict, expired/used hold, already cancelled), `SecurityException` (another user's hold/booking). Rust: `Ticketing` trait with `TicketError::{InvalidInput, Conflict, NotAllowed, Gone}`; `available_seats`, `seats_of`, `total_of` also return `Tix<..>` (unknown id is `InvalidInput`).
- A hold lives for exactly 5 minutes: at +4 minutes the seat is still unavailable, at +5 minutes it is available again and `confirm` of that hold fails (`IllegalStateException` / `Gone`).
- A hold for a show that has started (clock >= start) is refused with a conflict. `searchShows` stops listing a show once it has started.
- A seat label with an unknown row or a number beyond `seatsPerRow` (`Z9`, `A6` on a 5-seat row) is invalid input, not a conflict.
- `confirm` and `release` by another user: `SecurityException` / `NotAllowed`. A second `confirm` or `release` of the same hold, or `confirm` after the hold expired: `IllegalStateException` / `Gone`.
- Confirmed seats never come back by time; they only return through `cancel`. Cancel by another user is refused; cancelling twice is `IllegalStateException` / `Gone`.
- Refund thresholds use minutes until the show: `>= 120` full, `30..119` half rounded down, `< 30` (including already started) zero. The seats are freed in every case.
- `seatsOf` returns labels in row-then-number order (sorted); `availableSeats` shows 20 seats for a fresh 4x5 screen starting `A1..A5, B1 ...` and ending with `D5`.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Show times are minutes; "now" is the injected clock's `now_millis() / 60_000`. Seat labels are a row letter plus a
//! 1-based number: "A1", "A2", ..., "C10". Rows are A, B, C ... from the screen backwards; rows A and B are PREMIUM,
//! all other rows are REGULAR.

#[derive(Debug, PartialEq, Eq)]
pub enum TicketError {
    /// unknown show/screen/booking, bad seat label, empty or duplicate seats, non-positive duration/price
    InvalidInput,
    /// seat sold or held, show already started, or overlapping show on the same screen
    Conflict,
    /// another user's hold/booking
    NotAllowed,
    /// the hold expired or was already used/released, or the booking was already cancelled
    Gone,
}

pub type Tix<T> = Result<T, TicketError>;

pub trait Ticketing: Send + Sync {
    /// A screen with `rows` rows (1..=26) of `seats_per_row` seats. `InvalidInput` if it exists.
    fn add_screen(
        &self,
        theatre: &str,
        city: &str,
        screen: &str,
        rows: u32,
        seats_per_row: u32,
    ) -> Tix<()>;

    /// A show on a screen. `InvalidInput` for an unknown screen, duration 0, price 0 or duplicate show id;
    /// `Conflict` if it overlaps another show on the same screen (touching is fine).
    #[allow(clippy::too_many_arguments)]
    fn add_show(
        &self,
        show_id: &str,
        theatre: &str,
        screen: &str,
        movie: &str,
        start_minute: u64,
        duration_minutes: u64,
        regular_price: u64,
        premium_price: u64,
    ) -> Tix<()>;

    /// Ids of the shows of `movie` in `city` that have not started yet, sorted by start then id.
    fn search_shows(&self, city: &str, movie: &str) -> Vec<String>;

    /// Seats that can be held right now (not sold, not under a live hold), in row-then-number order.
    fn available_seats(&self, show_id: &str) -> Tix<Vec<String>>;

    /// Holds all the seats for the user for 5 minutes, all-or-nothing, and returns the hold id.
    fn hold(&self, show_id: &str, user: &str, seats: &[&str]) -> Tix<String>;

    /// Gives a hold up early.
    fn release(&self, hold_id: &str, user: &str) -> Tix<()>;

    /// Turns a live hold into a booking and returns the booking id.
    fn confirm(&self, hold_id: &str, user: &str) -> Tix<String>;

    /// Sorted seat labels of a booking.
    fn seats_of(&self, booking_id: &str) -> Tix<Vec<String>>;

    /// Sum of the seat prices (premium rows A,B at the premium price, the others at the regular price).
    fn total_of(&self, booking_id: &str) -> Tix<u64>;

    /// Cancels a booking and frees the seats; returns the refund: 100% of the total if the show starts in 120
    /// minutes or more, 50% (rounded down) if 30..119 minutes away, nothing if closer or already started.
    fn cancel(&self, booking_id: &str, user: &str) -> Tix<u64>;
}
```

Your wiring point (`src/lib.rs`):

```rust
/// All time-dependent behaviour (hold expiry, "not started yet", refunds) must use this clock.
pub fn create(_clock: Arc<dyn Clock>) -> Option<Arc<dyn Ticketing>> { ... }
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Time-dependent behaviour uses an injectable fake clock, never real sleeping. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Seat listing order, search by city and movie in time order (and dropping a show once started), overlapping shows refused but touching allowed, duplicate show id and unknown screen.
- Held seats disappear from `availableSeats`; a second hold on an overlapping seat fails and, being all-or-nothing, leaves the free seat free.
- Invalid holds: unknown show, unknown row `Z9`, number out of range `A6`, empty list, duplicate seats.
- Hold expiry at exactly 5 minutes (4 min still held), confirm after expiry fails, the seat can be taken by someone else.
- Confirm turns a hold into a booking with tiered prices (350+350+200) and seats sorted; confirmed seats never expire.
- Ownership: confirm/release by another user, double confirm, double release.
- Hold refused after the show started; refund at 600, 100 and 25 minutes before the show (350, 300, 0), seats come back, second cancel fails, other users cannot cancel.
- Concurrency: 16 threads hold one seat (exactly one wins); 16 threads randomly hold-and-confirm pairs of seats in a 200-seat show (no seat sold twice, available == 200 - sold); 8 threads hold the same two seats (exactly one winner, 18 seats left; repeated 100x).
- More edge cases (holds expiring while others keep booking, 2,000 shows) are in `interviewer/`.

## Language notes

A `Mutex<ShowState>` per show inside a `HashMap<String, Arc<Mutex<..>>>` (or one `Mutex` around everything to start); read time from the injected clock inside the guard, and use an `enum SeatState { Free, Held(HoldId), Sold(BookingId) }`.

## Running

```bash
cargo test -p p89_movie_ticketing
STRICT=1 cargo test -p p89_movie_ticketing   # fail instead of skip while unimplemented
cargo test -p p89_movie_ticketing --features hidden   # interviewer scenarios
cargo play run 89              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`FakeClock`, `sut!`): `rust/support`.

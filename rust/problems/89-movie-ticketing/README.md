# 89 · Movie Ticket Booking (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 75 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building the booking engine of a movie-ticketing app. Theatres have screens with seat grids, screens run shows (a movie at a start time), and customers search for a movie in their city, pick seats and pay. Because payment takes a while, picking seats must *hold* them for a short time (5 minutes) so nobody else can buy them in the meantime; if the customer does not confirm in time, the seats become available again by themselves. Premium rows cost more than regular rows. Customers can cancel a booking and get a refund that depends on how close the show is. On a Friday night thousands of people fight for the same seats, so a seat must never be sold twice.

## Glossary

- **Screen**: a room with `rows x seatsPerRow` seats. Row A is the first row; seat labels are the row letter plus a 1-based number (`A1`, `C10`).
- **Show**: one screening of a movie on a screen, occupying minutes `[start, start + duration)`.
- **Hold**: a temporary reservation of seats by one user. It lasts exactly 5 minutes from the moment it is made, then lapses.
- **Booking**: the result of confirming a live hold; its seats are sold.
- **Premium / regular**: rows A and B are premium, all other rows regular; each show has one price for each.
- **Now**: the injected clock's milliseconds divided by 60,000, in whole minutes.

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
Setup: screen s1 = 4 rows x 5 seats (rows A,B premium); show1 starts at minute 600, regular 200, premium 350. Clock at 0.

available_seats(show1)               -> 20 seats: A1..A5, B1..B5, C1..C5, D1..D5
hold(show1, ann, [B2, A1, C3])      -> holdId     available_seats now has 17 seats
hold(show1, bob, [A1, A2])          -> conflict error (A1 held); A2 is NOT left held, available_seats still contains A2
hold(show1, ann, ["A6"]) or ["Z9"]  -> invalid input (no such seat), not a conflict

Expiry
hold(show1, ann, [A1]) at minute 0; at minute 4 A1 is still unavailable;
at minute 5 A1 is available again, confirm(holdId, ann) fails (gone / IllegalStateException), bob can hold A1

Confirm and prices
confirm(holdId, ann)   -> bookingId
seats_of(bookingId)     -> [A1, B2, C3]
total_of(bookingId)     -> 350 + 350 + 200 = 900
Confirmed seats never return by waiting; only cancel frees them.

Refund (show at minute 600, booking A1 = 350, C1+C2+C3 = 600, D1 = 200)
cancel(A1 booking) at minute 0   (600 min away) -> 350 (100%)
cancel(C booking)  at minute 500 (100 min away) -> 300 (50% of 600)
cancel(D1 booking) at minute 575 (25 min away)  -> 0     seats are still freed
cancel(D1 booking) again                        -> error (already cancelled)
```

## Edge cases

- Refund thresholds by minutes until the show starts: `>= 120` full, `30..=119` half (rounded down, e.g. 50% of 351 = 175), `< 30` including already started: 0. The seats are freed in every case.
- A show that has started (`now >= start`) can no longer be held and is no longer returned by `search_shows`.
- `add_show` that overlaps another show on the same screen is a conflict (e.g. show1 runs 600..750, a new show at 700 conflicts, one at 750 is fine); a duplicate show id, unknown screen or a duration/price of 0 is invalid input.
- `hold` with an empty list, a duplicate seat in the list, an unknown show, or a non-existent seat is invalid input; a sold or currently held seat, or a started show, is a conflict.
- `confirm`/`release` by someone other than the holder: not allowed. A second `confirm`/`release` of the same hold, or `confirm` after expiry: gone. `cancel` by someone else: not allowed; `cancel` twice: gone.
- 16 threads holding the same seat at the same moment: exactly one succeeds, the rest get the conflict error.

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

## Running

```bash
cargo test -p p89_movie_ticketing
STRICT=1 cargo test -p p89_movie_ticketing   # fail instead of skip while unimplemented
cargo test -p p89_movie_ticketing --features hidden   # interviewer scenarios
cargo play run 89              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`FakeClock`, `sut!`): `rust/support`.

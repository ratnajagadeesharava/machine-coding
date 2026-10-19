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
    fn add_screen(&self, theatre: &str, city: &str, screen: &str, rows: u32, seats_per_row: u32) -> Tix<()>;

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

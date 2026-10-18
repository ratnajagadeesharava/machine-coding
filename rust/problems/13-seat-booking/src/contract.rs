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

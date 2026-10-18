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

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

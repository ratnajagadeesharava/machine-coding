//! Thin test entry point. Your domain model lives behind it.

pub trait RideSharingApi {
    fn add_driver(&mut self, driver_id: &str, x: i32, y: i32);

    /// Matches the nearest available driver within distance 10 and returns the ride id.
    fn request_ride(
        &mut self,
        rider_id: &str,
        pickup_x: i32,
        pickup_y: i32,
        drop_x: i32,
        drop_y: i32,
    ) -> Option<String>;

    fn driver_of(&self, ride_id: &str) -> Option<String>;

    /// MATCHED, IN_PROGRESS, COMPLETED, CANCELLED or UNKNOWN.
    fn status_of(&self, ride_id: &str) -> String;

    fn start_ride(&mut self, ride_id: &str) -> bool;

    /// Returns the fare.
    fn complete_ride(&mut self, ride_id: &str) -> Option<u64>;

    fn cancel_ride(&mut self, ride_id: &str) -> bool;

    /// "x,y" or `None`.
    fn driver_location(&self, driver_id: &str) -> Option<String>;
}

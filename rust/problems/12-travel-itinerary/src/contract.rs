//! Thin test entry point. Your domain model lives behind it.

pub trait ItineraryApi {
    fn create_trip(&mut self, trip_id: &str, start_day: i32, end_day: i32, budget: u64);

    /// Symmetric travel time between two locations; applies to future additions.
    fn set_travel_minutes(&mut self, location_a: &str, location_b: &str, minutes: u32);

    #[allow(clippy::too_many_arguments)]
    fn add_activity(
        &mut self,
        trip_id: &str,
        activity_id: &str,
        location: &str,
        day: i32,
        start_minute: i32,
        end_minute: i32,
        cost: i64,
    ) -> bool;

    fn remove_activity(&mut self, trip_id: &str, activity_id: &str) -> bool;

    /// Activity ids of the day ordered by start time.
    fn agenda(&self, trip_id: &str, day: i32) -> Vec<String>;

    fn total_cost(&self, trip_id: &str) -> u64;
}

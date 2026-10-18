//! Thin test entry point. Your domain model lives behind it.

pub trait ElevatorApi {
    /// Hall call. `direction` is "UP" or "DOWN". Returns false for invalid requests.
    fn request_pickup(&mut self, floor: i32, direction: &str) -> bool;

    /// Cabin button inside the given elevator (0-based). Returns false for invalid requests.
    fn request_destination(&mut self, elevator: usize, floor: i32) -> bool;

    /// Advances the simulation by one tick.
    fn step(&mut self);

    fn floor_of(&self, elevator: usize) -> i32;

    /// True only during the step in which the doors were opened.
    fn doors_open(&self, elevator: usize) -> bool;
}

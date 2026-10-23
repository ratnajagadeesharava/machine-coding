//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! A building has floors `0..floors`, one lift, a capacity in persons and a limit on intermediate stops.
//! The lift starts idle at floor 0 and only moves when `step` is called.

#[derive(Debug, PartialEq, Eq)]
pub enum LiftError {
    /// blank request id, id of a rider that is still waiting or onboard, floor outside the building,
    /// `from_floor == to_floor`, or `persons == 0`
    InvalidArgument,
}

pub trait LiftController: Send + Sync {
    /// Asks to be carried from `from_floor` to `to_floor`. `Ok(true)`: accepted, the promise is recorded.
    /// `Ok(false)`: rejected, the state is exactly as before the call.
    ///
    /// Accepted only if, when the lift follows its sweep (see the README) with the new request added, the load never
    /// exceeds the capacity and every rider (the new one and every rider accepted earlier) reaches their destination
    /// with at most the configured number of intermediate stops. A request for more persons than the capacity is
    /// rejected (`Ok(false)`).
    fn request(
        &self,
        request_id: &str,
        from_floor: u32,
        to_floor: u32,
        persons: u32,
    ) -> Result<bool, LiftError>;

    /// Moves the lift to its next stop, lets the riders destined for that floor off and the riders waiting there (in
    /// the sweep's direction) on. Returns the floor of that stop, or `None` (and does nothing) when no accepted
    /// rider is waiting or onboard.
    fn step(&self) -> Option<u32>;

    /// The floor the lift is at.
    fn current_floor(&self) -> u32;

    /// Total persons currently inside the lift.
    fn onboard(&self) -> u32;

    /// The floors of all upcoming stops, in the order the lift will make them, until every accepted rider is
    /// delivered. Empty when idle. Calling `step` repeatedly returns exactly these floors in this order
    /// (as long as nothing else is accepted in between).
    fn plan(&self) -> Vec<u32>;

    /// `"WAITING"`, `"ONBOARD"`, `"DELIVERED"`, or `"UNKNOWN"` (never accepted / rejected).
    fn status(&self, request_id: &str) -> String;
}

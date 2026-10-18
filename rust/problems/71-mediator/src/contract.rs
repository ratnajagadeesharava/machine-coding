//! Provided: the colleague trait and the error type.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TowerError {
    /// The aircraft is already on the runway or in the queue.
    Duplicate,
    /// `landed` for an aircraft that is not the one on the runway.
    NotOnRunway,
    /// The tower does not know that aircraft (or it is already cleared, for an emergency).
    Unknown,
}

/// An aircraft that is told what to do by the tower. Callbacks take `&self` (use interior mutability).
pub trait Aircraft {
    /// Unique call sign. Two aircraft with the same id are the same aircraft.
    fn id(&self) -> &str;
    /// The runway is yours: land now.
    fn on_cleared_to_land(&self);
    /// Hold and wait. `position` is your place in the landing queue (1 = next to land).
    fn on_hold(&self, position: usize);
}

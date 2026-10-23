//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Vocabulary: vehicle types are the strings `"TWO_WHEELER"` and `"FOUR_WHEELER"`; strategies are the strings
//! `"LOWEST_INDEX_SPOT"` and `"MOST_FREE_FLOOR"`. A spot is described as `"floor:spot"` (for example `"1:4"`)
//! where `spot` is the global spot index (see the README for the numbering).

#[derive(Debug, PartialEq, Eq)]
pub enum ParkingError {
    /// unknown vehicle type / strategy, blank registration or ticket id, floor out of range,
    /// invalid lot configuration
    InvalidArgument,
    /// a vehicle with this registration number is currently parked
    AlreadyParked,
}

pub trait ParkingLot: Send + Sync {
    /// Parks a vehicle in a free spot of its own type, chosen by the current strategy, and returns the ticket id.
    /// `Ok(None)` (and nothing changes) if no compatible spot is free. A registration that is currently parked is
    /// `Err(AlreadyParked)`, checked before looking for a spot, so it also applies when the lot is full.
    fn park(&self, vehicle_type: &str, registration: &str) -> Result<Option<String>, ParkingError>;

    /// Frees the spot held by the ticket. `Ok(true)` if the ticket was active; `Ok(false)` for an unknown ticket or
    /// one that was already used. The freed spot is available to the very next `park`.
    fn unpark(&self, ticket_id: &str) -> Result<bool, ParkingError>;

    /// "floor:spot" of the vehicle with this registration number, `Ok(None)` if it is not currently parked.
    fn find_by_registration(&self, registration: &str) -> Result<Option<String>, ParkingError>;

    /// "floor:spot" held by an active ticket, `Ok(None)` for an unknown or already used ticket.
    fn find_by_ticket(&self, ticket_id: &str) -> Result<Option<String>, ParkingError>;

    /// Number of free spots of this type in the whole lot.
    fn free_spots(&self, vehicle_type: &str) -> Result<usize, ParkingError>;

    /// Number of free spots of this type on one floor (`InvalidArgument` if the floor does not exist).
    fn free_spots_on_floor(&self, floor: usize, vehicle_type: &str) -> Result<usize, ParkingError>;

    /// Switches the allocation strategy for all later `park` calls; vehicles already parked do not move.
    fn set_strategy(&self, strategy: &str) -> Result<(), ParkingError>;
}

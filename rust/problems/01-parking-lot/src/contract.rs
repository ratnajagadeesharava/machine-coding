//! Thin test entry point. Your domain model lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

pub trait ParkingLotApi: Send + Sync {
    /// Parks the vehicle. Returns a ticket id, or `None` if refused.
    fn enter(&self, vehicle_type: &str, plate: &str) -> Option<String>;

    /// Releases the spot of the ticket and returns the fee, or `None` if the ticket is unknown/already used.
    fn exit(&self, ticket_id: &str) -> Option<u64>;

    /// Number of currently free spots that could accept this vehicle type.
    fn available_spots(&self, vehicle_type: &str) -> u32;

    /// 0-based floor of the spot held by an active ticket.
    fn floor_of(&self, ticket_id: &str) -> Option<u32>;
}

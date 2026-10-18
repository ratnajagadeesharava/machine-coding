//! Provided: the policy trait and the error type.

pub trait FeePolicy {
    /// Whether this policy prices the given vehicle type (case-insensitive).
    fn supports(&self, vehicle_type: &str) -> bool;
    /// Fee in cents for a stay of `minutes`.
    fn fee(&self, minutes: u32) -> u64;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeeError {
    /// No registered policy supports that vehicle type.
    UnknownVehicle,
}

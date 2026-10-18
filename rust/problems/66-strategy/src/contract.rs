//! Provided: the parcel, the strategy trait and the error type. Amounts are in cents.

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Parcel {
    pub weight_grams: u32,
    pub distance_km: u32,
    pub value_cents: u64,
}

/// One way of pricing delivery.
pub trait ShippingStrategy {
    /// Short label, e.g. "flat".
    fn name(&self) -> String;
    fn cost(&self, parcel: &Parcel) -> u64;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShipError {
    /// A parcel under 1 gram.
    InvalidParcel,
    /// `cheapest` was given no strategies.
    NoStrategies,
}

//! Strategy — interchangeable pricing algorithms behind the `ShippingStrategy` trait.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// name() is "flat". Every parcel costs `cents`.
pub struct FlatRate;

impl FlatRate {
    pub fn new(cents: u64) -> FlatRate {
        todo!()
    }
}

impl ShippingStrategy for FlatRate {
    fn name(&self) -> String {
        todo!()
    }
    fn cost(&self, parcel: &Parcel) -> u64 {
        todo!()
    }
}

/// name() is "weight". Cost = cents_per_kg * (number of STARTED kilograms): 1 g -> 1 kg, 1000 g -> 1 kg, 1001 g -> 2 kg.
pub struct ByWeight;

impl ByWeight {
    pub fn new(cents_per_kg: u64) -> ByWeight {
        todo!()
    }
}

impl ShippingStrategy for ByWeight {
    fn name(&self) -> String {
        todo!()
    }
    fn cost(&self, parcel: &Parcel) -> u64 {
        todo!()
    }
}

/// name() is "distance". Cost = cents_per_km * distance_km.
pub struct DistanceBased;

impl DistanceBased {
    pub fn new(cents_per_km: u64) -> DistanceBased {
        todo!()
    }
}

impl ShippingStrategy for DistanceBased {
    fn name(&self) -> String {
        todo!()
    }
    fn cost(&self, parcel: &Parcel) -> u64 {
        todo!()
    }
}

/// name() is `"free>=<min>/<fallback name>"`. Parcels worth at least `min_value_cents` ship free; others cost
/// whatever the fallback strategy says.
pub struct FreeOverThreshold;

impl FreeOverThreshold {
    pub fn new(min_value_cents: u64, fallback: Box<dyn ShippingStrategy>) -> FreeOverThreshold {
        todo!()
    }
}

impl ShippingStrategy for FreeOverThreshold {
    fn name(&self) -> String {
        todo!()
    }
    fn cost(&self, parcel: &Parcel) -> u64 {
        todo!()
    }
}

/// The context. The pricing algorithm is a replaceable strategy it is given.
pub struct ShippingCalculator;

impl ShippingCalculator {
    pub fn new(strategy: Box<dyn ShippingStrategy>) -> ShippingCalculator {
        todo!()
    }

    /// Swaps the algorithm at runtime.
    pub fn set_strategy(&mut self, strategy: Box<dyn ShippingStrategy>) {
        todo!()
    }

    pub fn strategy_name(&self) -> String {
        todo!()
    }

    /// The current strategy's cost. A parcel under 1 gram -> `Err(ShipError::InvalidParcel)`.
    pub fn quote(&self, parcel: &Parcel) -> Result<u64, ShipError> {
        todo!()
    }
}

/// Index of the cheapest strategy for this parcel; on a tie the one that comes first. Empty list ->
/// `Err(NoStrategies)`; a parcel under 1 gram -> `Err(InvalidParcel)`.
pub fn cheapest(strategies: &[Box<dyn ShippingStrategy>], parcel: &Parcel) -> Result<usize, ShipError> {
    todo!()
}

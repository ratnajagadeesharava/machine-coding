//! Strategy — interchangeable pricing algorithms behind the `ShippingStrategy` trait.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// Flat-rate shipping.
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

/// Weight-based shipping.
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

/// Distance-based shipping.
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

/// Free shipping above a parcel value, otherwise delegates to another strategy.
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

/// Quotes shipping for parcels.
pub struct ShippingCalculator;

impl ShippingCalculator {
    pub fn new(strategy: Box<dyn ShippingStrategy>) -> ShippingCalculator {
        todo!()
    }

    pub fn set_strategy(&mut self, strategy: Box<dyn ShippingStrategy>) {
        todo!()
    }

    pub fn strategy_name(&self) -> String {
        todo!()
    }

    /// The shipping cost for this parcel.
    pub fn quote(&self, parcel: &Parcel) -> Result<u64, ShipError> {
        todo!()
    }
}

/// Index of the cheapest strategy for this parcel.
pub fn cheapest(strategies: &[Box<dyn ShippingStrategy>], parcel: &Parcel) -> Result<usize, ShipError> {
    todo!()
}

//! Polymorphism — one trait, many behaviours. `TollPlaza` works with `&dyn Vehicle` only.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// 500 cents; peak 750.
pub struct Car;

impl Car {
    pub fn new(plate: &str) -> Car {
        todo!()
    }
}

impl Vehicle for Car {
    fn plate(&self) -> &str {
        todo!()
    }
    fn toll(&self, peak: bool) -> u64 {
        todo!()
    }
}

/// 200 cents, never surcharged.
pub struct Motorcycle;

impl Motorcycle {
    pub fn new(plate: &str) -> Motorcycle {
        todo!()
    }
}

impl Vehicle for Motorcycle {
    fn plate(&self) -> &str {
        todo!()
    }
    fn toll(&self, peak: bool) -> u64 {
        todo!()
    }
}

/// 1000 cents + 300 per axle beyond 2; peak = 1.5x. Fewer than 2 axles -> `TollError::InvalidAxles`.
pub struct Truck;

impl Truck {
    pub fn new(plate: &str, axles: u32) -> Result<Truck, TollError> {
        todo!()
    }
}

impl Vehicle for Truck {
    fn plate(&self) -> &str {
        todo!()
    }
    fn toll(&self, peak: bool) -> u64 {
        todo!()
    }
}

/// 800 cents; peak 960.
pub struct Bus;

impl Bus {
    pub fn new(plate: &str) -> Bus {
        todo!()
    }
}

impl Vehicle for Bus {
    fn plate(&self) -> &str {
        todo!()
    }
    fn toll(&self, peak: bool) -> u64 {
        todo!()
    }
}

/// Charges any `Vehicle`. It must not know which concrete types exist.
pub struct TollPlaza;

impl TollPlaza {
    pub fn new() -> TollPlaza {
        todo!()
    }

    /// Charges the vehicle's own toll, records the receipt "PLATE:amount" and returns the amount.
    pub fn charge(&mut self, vehicle: &dyn Vehicle, peak: bool) -> u64 {
        todo!()
    }

    pub fn total(&self) -> u64 {
        todo!()
    }

    /// One entry per charge, in order.
    pub fn receipts(&self) -> Vec<String> {
        todo!()
    }
}

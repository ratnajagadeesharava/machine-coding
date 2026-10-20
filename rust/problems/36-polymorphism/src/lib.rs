//! Polymorphism.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// Car.
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

/// Motorcycle.
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

/// Truck.
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

/// Bus.
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

/// Charges vehicles and keeps receipts.
pub struct TollPlaza;

impl TollPlaza {
    pub fn new() -> TollPlaza {
        todo!()
    }

    pub fn charge(&mut self, vehicle: &dyn Vehicle, peak: bool) -> u64 {
        todo!()
    }

    pub fn total(&self) -> u64 {
        todo!()
    }

    pub fn receipts(&self) -> Vec<String> {
        todo!()
    }
}

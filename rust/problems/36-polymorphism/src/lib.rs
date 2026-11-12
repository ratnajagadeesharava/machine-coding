//! Polymorphism — one trait, many behaviours. `TollPlaza` works with `&dyn Vehicle` only.
#![allow(unused_variables)]

pub mod contract;

use contract::*;
use std::fmt::format;

/// 500 cents; peak 750.
pub struct Car(String);

impl Car {
    pub fn new(plate: &str) -> Car {
        Car(plate.to_string())
    }
}

impl Vehicle for Car {
    fn plate(&self) -> &str {
        self.0.as_str()
    }
    fn toll(&self, peak: bool) -> u64 {
        if !peak {
            500
        } else {
            750
        }
    }
}

/// 200 cents, never surcharged.
pub struct Motorcycle(String);

impl Motorcycle {
    pub fn new(plate: &str) -> Motorcycle {
        Motorcycle(plate.to_string())
    }
}

impl Vehicle for Motorcycle {
    fn plate(&self) -> &str {
        self.0.as_str()
    }
    fn toll(&self, peak: bool) -> u64 {
        200
    }
}

/// 1000 cents + 300 per axle beyond 2; peak = 1.5x. Fewer than 2 axles -> `TollError::InvalidAxles`.
pub struct Truck(String, u32);

impl Truck {
    pub fn new(plate: &str, axles: u32) -> Result<Truck, TollError> {
        if axles < 2 {
            return Err(TollError::InvalidAxles);
        }
        Ok(Truck(plate.to_string(), axles))
    }
}

impl Vehicle for Truck {
    fn plate(&self) -> &str {
        self.0.as_str()
    }
    fn toll(&self, peak: bool) -> u64 {
        if !peak {
            1000 + 300 * (self.1 - 2) as u64
        } else {
            1500 + 450 * (self.1 - 2) as u64
        }
    }
}

/// 800 cents; peak 960.
pub struct Bus(String);

impl Bus {
    pub fn new(plate: &str) -> Bus {
        Bus(plate.to_string())
    }
}

impl Vehicle for Bus {
    fn plate(&self) -> &str {
        self.0.as_str()
    }
    fn toll(&self, peak: bool) -> u64 {
        if peak {
            960
        } else {
            800
        }
    }
}

/// Charges any `Vehicle`. It must not know which concrete types exist.
pub struct TollPlaza {
    total_charge: u64,
    receipts: Vec<String>,
}

impl TollPlaza {
    pub fn new() -> TollPlaza {
        TollPlaza {
            total_charge: 0,
            receipts: Vec::new(),
        }
    }

    /// Charges the vehicle's own toll, records the receipt "PLATE:amount" and returns the amount.
    pub fn charge(&mut self, vehicle: &dyn Vehicle, peak: bool) -> u64 {
        let cost = vehicle.toll(peak);
        self.total_charge = self.total_charge + cost;
        let s = format!("{}:{}", vehicle.plate(), cost);
        self.receipts.push(s);
        cost
    }

    pub fn total(&self) -> u64 {
        self.total_charge
    }

    /// One entry per charge, in order.
    pub fn receipts(&self) -> Vec<String> {
        self.receipts.clone()
    }
}

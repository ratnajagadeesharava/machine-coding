//! The built-in policies.
#![allow(unused_variables)]

use crate::contract::FeePolicy;

/// Parking fee policy for cars.
pub struct CarPolicy;

impl CarPolicy {
    pub fn new() -> CarPolicy {
        todo!()
    }
}

impl FeePolicy for CarPolicy {
    fn supports(&self, vehicle_type: &str) -> bool {
        todo!()
    }
    fn fee(&self, minutes: u32) -> u64 {
        todo!()
    }
}

/// Parking fee policy for motorcycles.
pub struct MotorcyclePolicy;

impl MotorcyclePolicy {
    pub fn new() -> MotorcyclePolicy {
        todo!()
    }
}

impl FeePolicy for MotorcyclePolicy {
    fn supports(&self, vehicle_type: &str) -> bool {
        todo!()
    }
    fn fee(&self, minutes: u32) -> u64 {
        todo!()
    }
}

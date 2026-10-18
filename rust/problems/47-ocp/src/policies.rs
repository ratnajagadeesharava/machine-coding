//! The built-in policies. Keep them in this file: the tests look at it.
#![allow(unused_variables)]

use crate::contract::FeePolicy;

/// Supports "car" (any letter case). 0 minutes -> 0. Otherwise 200 cents for the first 60 minutes plus 100 cents
/// for every started 30 minutes after that, never more than 1500.
/// Examples: 30 -> 200, 60 -> 200, 61 -> 300, 90 -> 300, 91 -> 400, 1000 -> 1500.
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

/// Supports "motorcycle" (any letter case). Half of what a car would pay (integer division).
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

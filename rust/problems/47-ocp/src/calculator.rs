//! The calculator. It must contain NO knowledge of any vehicle type. Keep it in this file: the tests look at it.
#![allow(unused_variables)]

use crate::contract::{FeeError, FeePolicy};

pub struct FeeCalculator;

impl FeeCalculator {
    pub fn new(policies: Vec<Box<dyn FeePolicy>>) -> FeeCalculator {
        todo!()
    }

    /// Adds a policy. When several policies support a type, the most recently registered one wins.
    pub fn register(&mut self, policy: Box<dyn FeePolicy>) {
        todo!()
    }

    /// Fee from the winning policy; no policy supports the type -> `Err(FeeError::UnknownVehicle)`.
    pub fn fee(&self, vehicle_type: &str, minutes: u32) -> Result<u64, FeeError> {
        todo!()
    }
}

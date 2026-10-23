//! The fee calculator.
#![allow(unused_variables)]

use crate::contract::{FeeError, FeePolicy};

pub struct FeeCalculator;

impl FeeCalculator {
    pub fn new(policies: Vec<Box<dyn FeePolicy>>) -> FeeCalculator {
        todo!()
    }

    pub fn register(&mut self, policy: Box<dyn FeePolicy>) {
        todo!()
    }

    pub fn fee(&self, vehicle_type: &str, minutes: u32) -> Result<u64, FeeError> {
        todo!()
    }
}

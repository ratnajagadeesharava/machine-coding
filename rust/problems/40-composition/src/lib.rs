//! Composition.
#![allow(unused_variables)]

pub mod contract;

use contract::{LineItem, OrderError};

pub struct Order;

impl Order {
    pub fn new(id: &str) -> Order {
        todo!()
    }

    pub fn id(&self) -> &str {
        todo!()
    }

    pub fn add_line(&mut self, sku: &str, quantity: u32, unit_price: u64) -> Result<(), OrderError> {
        todo!()
    }

    pub fn lines(&self) -> &[LineItem] {
        todo!()
    }

    pub fn remove_line(&mut self, sku: &str) -> bool {
        todo!()
    }

    pub fn total(&self) -> u64 {
        todo!()
    }

    pub fn cancel(&mut self) {
        todo!()
    }

    pub fn is_cancelled(&self) -> bool {
        todo!()
    }
}

//! Encapsulation.
#![allow(unused_variables)]

pub mod contract;

use contract::StockError;

/// A warehouse stock record.
pub struct StockItem;

impl StockItem {
    pub fn new(sku: &str, capacity: u32) -> Result<StockItem, StockError> {
        todo!()
    }

    pub fn sku(&self) -> &str {
        todo!()
    }

    pub fn capacity(&self) -> u32 {
        todo!()
    }

    pub fn quantity(&self) -> u32 {
        todo!()
    }

    pub fn add(&mut self, n: u32) -> Result<(), StockError> {
        todo!()
    }

    pub fn remove(&mut self, n: u32) -> Result<(), StockError> {
        todo!()
    }

    pub fn transfer_to(&mut self, other: &mut StockItem, n: u32) -> Result<(), StockError> {
        todo!()
    }

    pub fn add_tag(&mut self, tag: &str) {
        todo!()
    }

    pub fn tags(&self) -> &[String] {
        todo!()
    }

    pub fn history(&self) -> &[String] {
        todo!()
    }
}

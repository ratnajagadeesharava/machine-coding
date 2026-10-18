//! Encapsulation — fields stay private (no `pub` fields!); the only way in is through checked methods.
#![allow(unused_variables)]

pub mod contract;

use contract::StockError;

/// A warehouse stock record. Invariants: 0 <= quantity <= capacity, unique lower-case tags.
pub struct StockItem;

impl StockItem {
    /// Blank sku or zero capacity -> `InvalidInput`. Starts empty.
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

    /// n == 0 -> `InvalidInput`; above capacity -> `InvariantViolation` (nothing changes).
    pub fn add(&mut self, n: u32) -> Result<(), StockError> {
        todo!()
    }

    /// n == 0 -> `InvalidInput`; more than in stock -> `InvariantViolation` (nothing changes).
    pub fn remove(&mut self, n: u32) -> Result<(), StockError> {
        todo!()
    }

    /// Atomic: if either side would break its rules, neither changes.
    pub fn transfer_to(&mut self, other: &mut StockItem, n: u32) -> Result<(), StockError> {
        todo!()
    }

    /// Trims and lower-cases; blank and duplicate tags are ignored.
    pub fn add_tag(&mut self, tag: &str) {
        todo!()
    }

    /// Tags in insertion order.
    pub fn tags(&self) -> &[String] {
        todo!()
    }

    /// One entry per successful change: "+5", "-2", "-1 (transfer)" (source), "+1 (transfer)" (target).
    pub fn history(&self) -> &[String] {
        todo!()
    }
}

//! Composition — an `Order` OWNS its `LineItem`s (a `Vec<LineItem>` by value). When the order is dropped, so are they.
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

    /// Creates the line INSIDE the order (use `LineItem::new`, which only this crate can call).
    /// Blank sku / zero quantity / zero price -> `InvalidLine`. Same SKU again -> adds to that line's quantity
    /// (`LineItem::add_quantity`); a different unit price -> `PriceMismatch`. Cancelled -> `Cancelled`.
    pub fn add_line(&mut self, sku: &str, quantity: u32, unit_price: u64) -> Result<(), OrderError> {
        todo!()
    }

    pub fn lines(&self) -> &[LineItem] {
        todo!()
    }

    /// Removes and destroys the line for that SKU. Returns whether it existed.
    pub fn remove_line(&mut self, sku: &str) -> bool {
        todo!()
    }

    pub fn total(&self) -> u64 {
        todo!()
    }

    /// Destroys all lines and blocks further `add_line` calls.
    pub fn cancel(&mut self) {
        todo!()
    }

    pub fn is_cancelled(&self) -> bool {
        todo!()
    }
}

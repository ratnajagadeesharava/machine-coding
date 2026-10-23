//! Provided: the part (`LineItem`) and the error type.

use std::sync::atomic::{AtomicUsize, Ordering};

static LIVE: AtomicUsize = AtomicUsize::new(0);

pub struct LineItem {
    sku: String,
    quantity: u32,
    unit_price: u64,
}

impl LineItem {
    pub(crate) fn new(sku: &str, quantity: u32, unit_price: u64) -> LineItem {
        LIVE.fetch_add(1, Ordering::SeqCst);
        LineItem { sku: sku.to_string(), quantity, unit_price }
    }

    pub(crate) fn add_quantity(&mut self, extra: u32) {
        self.quantity += extra;
    }

    pub fn sku(&self) -> &str {
        &self.sku
    }

    pub fn quantity(&self) -> u32 {
        self.quantity
    }

    pub fn unit_price(&self) -> u64 {
        self.unit_price
    }

    pub fn subtotal(&self) -> u64 {
        self.quantity as u64 * self.unit_price
    }

    /// Number of `LineItem`s currently alive in the whole process.
    pub fn live_count() -> usize {
        LIVE.load(Ordering::SeqCst)
    }
}

impl Drop for LineItem {
    fn drop(&mut self) {
        LIVE.fetch_sub(1, Ordering::SeqCst);
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderError {
    /// Blank sku, zero quantity or zero price.
    InvalidLine,
    /// Same SKU added with a different unit price.
    PriceMismatch,
    /// The order was cancelled.
    Cancelled,
}

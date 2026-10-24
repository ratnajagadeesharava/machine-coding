//! Concurrent Bloom Filter - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports, unused_variables)]

pub mod contract;

use contract::*;
use std::sync::Arc;

/// `expected_items` must be at least 1 and `0 < false_positive_rate < 1`: `create` returns `None` otherwise.
pub fn create(expected_items: usize, false_positive_rate: f64) -> Option<Arc<dyn ConcurrentBloomFilter>> {
    None
}

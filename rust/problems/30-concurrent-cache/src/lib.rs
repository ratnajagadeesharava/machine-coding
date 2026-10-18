//! Concurrent Cache with Loaders — your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports)]

pub mod contract;

use contract::*;
use playground_support::Clock;
use std::sync::Arc;

pub fn create(
    _capacity: usize,
    _ttl_millis: u64,
    _clock: Arc<dyn Clock>,
) -> Option<Arc<dyn CacheApi>> {
    None
}

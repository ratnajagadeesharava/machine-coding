//! Search Result Cache - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports, unused_variables)]

pub mod contract;

use contract::*;
use playground_support::Clock;
use std::sync::Arc;

/// `capacity` (entries) and `ttl_millis` must be greater than 0: `create` returns `None` otherwise.
pub fn create(
    backend: Arc<dyn SearchBackend>,
    capacity: usize,
    ttl_millis: u64,
    clock: Arc<dyn Clock>,
) -> Option<Arc<dyn SearchCache>> {
    None
}

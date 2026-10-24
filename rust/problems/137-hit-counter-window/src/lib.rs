//! Configurable Time Window Hit Counter - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports, unused_variables)]

pub mod contract;

use contract::*;
use playground_support::Clock;
use std::sync::Arc;

/// `window_ms` must be at least 1: `create` returns `None` otherwise.
pub fn create(window_ms: u64, clock: Arc<dyn Clock>) -> Option<Arc<dyn HitCounter>> {
    None
}

//! Browser History Manager - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports, unused_variables)]

pub mod contract;

use contract::*;
use playground_support::Clock;
use std::sync::Arc;

/// `capacity` (maximum number of global history entries) must be at least 1: `create` returns `None` otherwise.
pub fn create(capacity: usize, clock: Arc<dyn Clock>) -> Option<Arc<dyn BrowserHistory>> {
    None
}

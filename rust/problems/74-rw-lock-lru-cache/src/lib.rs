//! Concurrent LRU Cache (Read-Write Lock) - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports)]

pub mod contract;

use contract::*;
use std::sync::Arc;

/// `capacity` is at least 1.
pub fn create(_capacity: usize) -> Option<Arc<dyn SharedCache>> {
    None
}

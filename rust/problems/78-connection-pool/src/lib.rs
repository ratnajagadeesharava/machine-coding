//! Bounded Connection Pool - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports)]

pub mod contract;

use contract::*;
use std::sync::Arc;

/// `size` is at least 1.
pub fn create(_size: usize) -> Option<Arc<dyn Pool>> {
    None
}

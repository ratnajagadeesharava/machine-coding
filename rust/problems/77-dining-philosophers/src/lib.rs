//! Dining Philosophers - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports)]

pub mod contract;

use contract::*;
use std::sync::Arc;

/// `seats` is at least 2.
pub fn create(_seats: usize) -> Option<Arc<dyn Table>> {
    None
}

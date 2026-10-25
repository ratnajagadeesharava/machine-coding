//! Custom HashMap - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports, unused_variables)]

pub mod contract;

use contract::*;
use std::sync::Arc;

pub fn create(
    initial_capacity: usize,
    load_factor_percent: u32,
    hasher: Arc<dyn KeyHasher>,
) -> Option<Result<Arc<dyn StringMap>, MapError>> {
    None
}

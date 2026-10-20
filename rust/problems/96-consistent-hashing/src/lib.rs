//! Consistent Hash Ring - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports)]

pub mod contract;

use contract::*;
use std::sync::Arc;

/// `virtual_nodes` points per unit of weight on the ring (at least 1).
pub fn create(_virtual_nodes: usize) -> Option<Arc<dyn HashRing>> {
    None
}

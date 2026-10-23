//! Elevator Request Feasibility - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports)]

pub mod contract;

use contract::*;
use std::sync::Arc;

pub fn create(
    floors: u32,
    capacity: u32,
    max_intermediate_stops: u32,
) -> Option<Arc<dyn LiftController>> {
    None
}

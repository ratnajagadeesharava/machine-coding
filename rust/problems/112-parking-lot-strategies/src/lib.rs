//! Parking Lot with Allocation Strategies - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports)]

pub mod contract;

use contract::*;
use std::sync::Arc;

/// `two_wheeler_spots[f]` / `four_wheeler_spots[f]` are the spot counts on floor `f`.
/// Invalid configuration (different lengths, no floors, unknown strategy) is `Err(InvalidArgument)`.
pub fn create(
    two_wheeler_spots: &[usize],
    four_wheeler_spots: &[usize],
    strategy: &str,
) -> Option<Result<Arc<dyn ParkingLot>, ParkingError>> {
    None
}

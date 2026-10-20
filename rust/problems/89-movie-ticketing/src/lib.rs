//! Movie Ticketing (BookMyShow-style) - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports)]

pub mod contract;

use contract::*;
use playground_support::Clock;
use std::sync::Arc;

/// All time-dependent behaviour (hold expiry, "not started yet", refunds) must use this clock.
pub fn create(_clock: Arc<dyn Clock>) -> Option<Arc<dyn Ticketing>> {
    None
}

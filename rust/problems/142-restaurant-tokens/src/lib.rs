//! Restaurant Customer Tokens - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports)]

pub mod contract;

use contract::*;
use playground_support::Clock;
use std::sync::Arc;

pub fn create(
    _clock: Arc<dyn Clock>,
    _grace_time_ms: u64,
    _recall_window_ms: u64,
) -> Option<Arc<dyn TokenSystem>> {
    None
}

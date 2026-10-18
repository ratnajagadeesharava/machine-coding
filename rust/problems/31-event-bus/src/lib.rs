//! In-Process Event Bus — your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports)]

pub mod contract;

use contract::*;
use playground_support::Clock;
use std::sync::Arc;

pub fn create() -> Option<Arc<dyn EventBusApi>> {
    None
}

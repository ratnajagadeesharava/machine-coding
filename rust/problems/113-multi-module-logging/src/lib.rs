//! Multi-Module Logging Framework - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports)]

pub mod contract;

use contract::*;
use std::sync::Arc;

/// The service writes FILE logs to `file`, TOOL logs to `tool`; the shared buffer holds `buffer_capacity` logs.
pub fn create(
    file: Arc<dyn Sink>,
    tool: Arc<dyn Sink>,
    buffer_capacity: usize,
) -> Option<Arc<dyn LogService>> {
    None
}

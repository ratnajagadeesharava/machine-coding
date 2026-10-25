//! Alert Monitoring System - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports, unused_variables)]

pub mod contract;

use contract::*;
use playground_support::Clock;
use std::sync::Arc;

pub fn create(notifier: Arc<dyn Notifier>, clock: Arc<dyn Clock>) -> Option<Arc<dyn AlertMonitor>> {
    None
}

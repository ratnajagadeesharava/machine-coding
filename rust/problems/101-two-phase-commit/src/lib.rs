//! Two-Phase Commit Coordinator - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from the two factories below.
#![allow(unused_imports)]

pub mod contract;

use contract::*;
use playground_support::SendFn;
use std::sync::Arc;

/// A coordinator for the given participants. `log` may already contain records (restart).
pub fn create_coordinator(_id: &str, _participants: &[&str], _send: SendFn, _log: Log) -> Option<Arc<dyn CoordinatorNode>> {
    None
}

/// A participant that reports to `coordinator_id`. `log` may already contain records (restart).
pub fn create_participant(_id: &str, _coordinator_id: &str, _send: SendFn, _log: Log) -> Option<Arc<dyn ParticipantNode>> {
    None
}

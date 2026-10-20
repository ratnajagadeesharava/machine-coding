//! Quorum Key-Value Store - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports)]

pub mod contract;

use contract::*;
use playground_support::SendFn;
use std::sync::Arc;

/// * `id`: this node's name
/// * `cluster`: every node's name INCLUDING this one
/// * `send`: `send(to, payload)` delivers a message to another node (later, maybe never)
pub fn create(_id: &str, _cluster: &[&str], _send: SendFn) -> Option<Arc<dyn KvNode>> {
    None
}

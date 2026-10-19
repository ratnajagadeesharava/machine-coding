//! Leader Election (Term-Based) - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports)]

pub mod contract;

use contract::*;
use playground_support::SendFn;
use std::sync::Arc;

/// * `id`: this node's name
/// * `peers`: the OTHER nodes' names (the cluster has `peers.len() + 1` nodes)
/// * `send`: `send(to, payload)` delivers a message to another node (later, maybe never)
/// * `seed`: seed for the election-timeout random numbers
pub fn create(_id: &str, _peers: &[&str], _send: SendFn, _seed: u64) -> Option<Arc<dyn ElectionNode>> {
    None
}

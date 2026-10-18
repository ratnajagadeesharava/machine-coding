//! Snake and Ladder — your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports)]

pub mod contract;

use contract::*;
use playground_support::Clock;
use std::sync::Arc;

pub fn create(
    _size: u32,
    _snakes: &[(u32, u32)],
    _ladders: &[(u32, u32)],
    _players: &[&str],
) -> Option<Box<dyn SnakeAndLadderApi>> {
    None
}

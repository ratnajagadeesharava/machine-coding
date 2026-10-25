//! Ludo - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports, unused_variables)]

pub mod contract;

use contract::*;

/// New game for `players` (2..=4) players that consumes `dice` (values 1..=6) in order.
/// `InvalidArgument` for a player count outside 2..=4 or a die outside 1..=6.
pub fn create(players: usize, dice: &[u8]) -> Option<Result<Box<dyn Ludo>, LudoError>> {
    None
}

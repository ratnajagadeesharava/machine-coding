//! Snake Game With Food And Score - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports)]

pub mod contract;

use contract::*;

/// `foods` are (row, col) cells in spawn order. `None` means "not implemented yet";
/// `Some(Err(..))` is an invalid board or food list.
pub fn create(
    width: i32,
    height: i32,
    foods: &[(i32, i32)],
) -> Option<Result<Box<dyn SnakeGame>, GameError>> {
    let _ = (width, height, foods);
    None
}

//! Text Editor with Grouped Undo/Redo - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports)]

pub mod contract;

use contract::*;

/// An empty document, cursor at 0.
pub fn create() -> Option<Box<dyn Editor>> {
    None
}

//! Document Line Search - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports)]

pub mod contract;

use contract::*;
use std::sync::Arc;

pub fn create<S: AsRef<str>>(_lines: &[S]) -> Option<Arc<dyn DocumentSearch>> {
    None
}

//! Document Comment Service - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports)]

pub mod contract;

use contract::*;
use std::sync::Arc;

pub fn create(_document_ids: &[&str]) -> Option<Arc<dyn CommentService>> {
    None
}

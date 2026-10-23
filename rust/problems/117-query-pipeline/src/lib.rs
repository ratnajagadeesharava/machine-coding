//! Search Query Pipeline - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` below.
#![allow(unused_imports, unused_variables)]

pub mod contract;

use contract::*;
use std::sync::Arc;

/// `dictionary`: correct words (lower-case) for the spell step. `synonyms`: lower-case word -> its synonyms
/// (lower-case, single words). `banned`: lower-case words the filter step removes. `corpus`: (doc id, text) pairs
/// searched by the rank step.
pub fn create(
    dictionary: &[&str],
    synonyms: &[(&str, &[&str])],
    banned: &[&str],
    corpus: &[(&str, &str)],
) -> Option<Arc<dyn QueryPipeline>> {
    None
}

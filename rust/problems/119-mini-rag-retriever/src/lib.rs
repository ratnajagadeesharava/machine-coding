//! Mini RAG Retrieval Component - your design lives in this crate.
//!
//! `contract` is the thin test entry point. Add your own modules, structs, enums and traits,
//! then return them from `create` / `create_with_store` below.
#![allow(unused_imports, unused_variables)]

pub mod contract;

use contract::*;
use std::sync::Arc;

/// Engine with your own default vector store; default chunking is 100 words with an overlap of 20.
pub fn create(embedder: Arc<dyn Embedder>) -> Option<Arc<dyn RagEngine>> {
    None
}

/// Same engine, but vectors go to the given store.
pub fn create_with_store(
    embedder: Arc<dyn Embedder>,
    store: Arc<dyn VectorStore>,
) -> Option<Arc<dyn RagEngine>> {
    None
}

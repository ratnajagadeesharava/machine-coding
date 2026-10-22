//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

/// An embedding service failure (the tests' fake embedder can be told to fail).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EmbedError(pub String);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RagError {
    /// blank doc id / blank query / k = 0 / invalid chunking
    InvalidArgument,
    /// the embedder failed; nothing was changed
    Embedding(String),
}

/// Turns text into a vector. Implemented by the tests (deterministic fakes); your component only calls it.
/// Every vector from one embedder has the same length.
pub trait Embedder: Send + Sync {
    fn embed(&self, text: &str) -> Result<Vec<f64>, EmbedError>;
}

/// One search result: the stored id, its cosine similarity to the query, and the payload stored with it.
#[derive(Debug, Clone, PartialEq)]
pub struct Hit {
    pub id: String,
    pub score: f64,
    pub payload: String,
}

/// Stores vectors and finds the ones most similar to a query. The tests supply an alternative implementation
/// (a plain brute-force scan) to prove the store can be swapped; your component must also work without one.
/// Implementations are thread-safe.
pub trait VectorStore: Send + Sync {
    /// Inserts the vector, or replaces the vector and payload already stored under `id`.
    fn upsert(&self, id: &str, vector: &[f64], payload: &str);

    /// Removes the entry; returns true if it existed.
    fn remove(&self, id: &str) -> bool;

    /// At most `k` hits (k >= 1), most similar first by cosine similarity; equal scores are ordered by id
    /// ascending (byte-wise string order). A zero vector (query or stored) has similarity 0 with everything.
    fn top_k(&self, query: &[f64], k: usize) -> Vec<Hit>;
}

pub trait RagEngine: Send + Sync {
    /// Splits `text` on whitespace into words and cuts it into chunks of at most chunk_size words, consecutive
    /// chunks sharing `overlap` words (see README). Stores the chunks under ids `"{doc_id}#{n}"` (n from 0) and
    /// returns how many were created. Empty or blank text creates 0 chunks. Re-ingesting a doc id replaces all
    /// its previous chunks as one atomic step. `InvalidArgument` for a blank doc id. If the embedder fails the
    /// call returns `Embedding(..)` and nothing changes.
    fn ingest(&self, doc_id: &str, text: &str) -> Result<usize, RagError>;

    /// Chunk size and overlap (in words) used by LATER ingests; chunks already stored are untouched.
    /// `InvalidArgument` unless chunk_size >= 1 and overlap < chunk_size (state unchanged then).
    fn set_chunking(&self, chunk_size: usize, overlap: usize) -> Result<(), RagError>;

    /// Removes every chunk of the document; Ok(false) if the document was unknown. A blank doc id is an error.
    fn delete_document(&self, doc_id: &str) -> Result<bool, RagError>;

    /// Ids of the `min(k, chunk_count())` chunks most similar (cosine) to the query, best first, ties by chunk id
    /// ascending (byte-wise string order). `InvalidArgument` for k = 0 or a blank query.
    fn retrieve(&self, query: &str, k: usize) -> Result<Vec<String>, RagError>;

    /// Prompt for the language model; the exact template is in the README. Uses the chunks
    /// `retrieve(query, k)` would return, whole chunks only, within a context budget of `max_context_chars`.
    /// `InvalidArgument` for k = 0 or a blank query.
    fn build_prompt(
        &self,
        query: &str,
        k: usize,
        max_context_chars: usize,
    ) -> Result<String, RagError>;

    /// Number of chunks currently stored.
    fn chunk_count(&self) -> usize;

    /// The text of a stored chunk, or `None` if there is no such chunk.
    fn chunk_text(&self, chunk_id: &str) -> Option<String>;
}

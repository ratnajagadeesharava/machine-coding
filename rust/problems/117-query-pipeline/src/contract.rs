//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

use std::sync::Arc;

/// Names of the built-in steps, in their fixed default order.
pub const PARSE: &str = "parse";
pub const SPELL: &str = "spell";
pub const SYNONYMS: &str = "synonyms";
pub const FILTER: &str = "filter";
pub const RANK: &str = "rank";
pub const HIGHLIGHT: &str = "highlight";

#[derive(Debug, PartialEq, Eq)]
pub enum PipelineError {
    /// blank query, null-equivalent step, blank/duplicate step name, unknown or non-transforming anchor,
    /// unknown step name
    InvalidArgument,
}

/// A query-transforming step supplied by the caller (the tests implement it). `name()` is constant for the life of
/// the step. `transform` receives the current query text and returns the new one. `Err(_)` means the step failed
/// (the Rust counterpart of throwing).
pub trait Step: Send + Sync {
    fn name(&self) -> String;
    fn transform(&self, query: &str) -> Result<String, String>;
}

/// What one `process` call produced.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryResult {
    /// the query text after the last query-transforming step that ran ("" when it became empty)
    pub query: String,
    /// the ranked, highlighted result lines, best first; empty when nothing matched
    pub results: Vec<String>,
    /// names of the steps that changed something, in execution order; a failed step appears as "<name>:failed"
    pub trace: Vec<String>,
}

pub trait QueryPipeline: Send + Sync {
    /// Runs the pipeline on a raw query. The query-transforming steps are parse, spell, synonyms and filter (plus
    /// any registered steps) in their current order, then rank, then highlight. `InvalidArgument` when `raw_query`
    /// is blank (nothing is run). Each call is independent; the returned vectors are never affected by later calls.
    fn process(&self, raw_query: &str) -> Result<QueryResult, PipelineError>;

    /// Adds a step at the end of the query-transforming phase (after "filter" and after earlier registered steps,
    /// before rank). `InvalidArgument` when the step's name is blank or a step with the same name exists (built-in
    /// names are always taken, even after the built-in was removed). Nothing changes on error.
    fn register(&self, step: Arc<dyn Step>) -> Result<(), PipelineError>;

    /// Adds a step immediately before the named query-transforming step (parse, spell, synonyms, filter or a
    /// registered step). `InvalidArgument` for the same reasons as `register`, or when the anchor is unknown,
    /// removed, or not a query-transforming step ("rank", "highlight"). Nothing changes on error.
    fn register_before(&self, step_name: &str, step: Arc<dyn Step>) -> Result<(), PipelineError>;

    /// Like `register_before` but immediately after the named step.
    fn register_after(&self, step_name: &str, step: Arc<dyn Step>) -> Result<(), PipelineError>;

    /// Permanently removes a step (built-in or registered); it no longer runs. Removing "rank" leaves results
    /// empty; removing "highlight" leaves the result lines unhighlighted. `InvalidArgument` for an unknown name.
    fn remove_step(&self, name: &str) -> Result<(), PipelineError>;

    /// Keeps the step but skips it (not run, not in the trace) until enabled. `InvalidArgument` for an unknown name.
    fn disable_step(&self, name: &str) -> Result<(), PipelineError>;

    /// Undoes `disable_step`; a no-op for a step that is already enabled. `InvalidArgument` for an unknown name.
    fn enable_step(&self, name: &str) -> Result<(), PipelineError>;
}

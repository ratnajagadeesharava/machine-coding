//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

/// `parallelism` was 0.
#[derive(Debug, PartialEq, Eq)]
pub struct InvalidParallelism;

pub trait ParallelSorter: Send + Sync {
    /// Sorts `data` in place, ascending, splitting the work over at most `parallelism` worker threads
    /// (the caller may take part). `parallelism == 1` must not need any extra thread.
    /// `Err(InvalidParallelism)` if `parallelism == 0`. Any length (including 0 and 1) and any values are valid.
    fn sort(&self, data: &mut [i32], parallelism: usize) -> Result<(), InvalidParallelism>;
}

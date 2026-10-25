//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Built-in strategy names (case-sensitive): ROW_MAJOR, COLUMN_MAJOR, SNAKE, SPIRAL, SPIRAL_COUNTER, DIAGONAL, ZIGZAG,
//! BOUSTROPHEDON_COLUMNS, REVERSE_ROW_MAJOR (exact orders in the README).

use std::sync::Arc;

#[derive(Debug, PartialEq, Eq)]
pub enum TraversalError {
    /// zero rows or zero columns, or rows of different lengths
    InvalidMatrix,
    /// the strategy name is not built in and not registered
    UnknownStrategy,
    /// registering a name that is built in or already registered
    DuplicateStrategy,
    /// blank strategy name when registering, `k < 0`, `kth_value` with `k >= rows * cols`
    InvalidArgument,
    /// the cursor id was never handed out, or the cursor was closed
    UnknownCursor,
    /// a registered strategy returned a cell outside the matrix, or one that was already visited (traverse only);
    /// carries the 0-based index of the first offending visit
    InvalidPosition(u64),
}

/// A read-only rows x cols grid of ints. PROVIDED: the tests implement it (some of them over matrices far too big to
/// hold in memory, and they count the calls to `get`). Implementations are immutable and thread-safe.
pub trait MatrixSource: Send + Sync {
    fn rows(&self) -> usize;
    fn cols(&self) -> usize;
    /// The value at (r, c), `r < rows()`, `c < cols()`.
    fn get(&self, r: usize, c: usize) -> i32;
}

/// A visiting order, as a pure function from "which visit is this" to "which cell". PROVIDED: the tests implement it
/// and register their own instances.
pub trait TraversalStrategy: Send + Sync {
    /// The cell `(row, col)` visited by the `index`-th visit (0-based) of a `rows x cols` matrix.
    /// Called with `index < rows * cols`. A correct strategy maps those indexes onto every cell exactly once.
    /// The same arguments always give the same answer.
    fn position(&self, rows: usize, cols: usize, index: u64) -> (usize, usize);
}

/// PROVIDED convenience: an owned, immutable copy of a grid.
pub struct GridSource {
    cells: Vec<Vec<i32>>,
}

impl GridSource {
    /// `InvalidMatrix` if there are no rows, a row is empty, or the rows differ in length. The input is copied.
    pub fn new(grid: &[Vec<i32>]) -> Result<GridSource, TraversalError> {
        if grid.is_empty()
            || grid[0].is_empty()
            || grid.iter().any(|row| row.len() != grid[0].len())
        {
            return Err(TraversalError::InvalidMatrix);
        }
        Ok(GridSource {
            cells: grid.to_vec(),
        })
    }
}

impl MatrixSource for GridSource {
    fn rows(&self) -> usize {
        self.cells.len()
    }
    fn cols(&self) -> usize {
        self.cells[0].len()
    }
    fn get(&self, r: usize, c: usize) -> i32 {
        self.cells[r][c]
    }
}

pub trait MatrixTraversal: Send + Sync {
    /// All values in the order of the strategy. `UnknownStrategy` for an unknown name.
    fn traverse(&self, strategy: &str) -> Result<Vec<i32>, TraversalError>;

    /// Makes `strategy` usable by name in every operation of this service. `InvalidArgument` for a blank name,
    /// `DuplicateStrategy` if the name is built in or already registered.
    fn register_strategy(
        &self,
        name: &str,
        strategy: Box<dyn TraversalStrategy>,
    ) -> Result<(), TraversalError>;

    /// Opens a cursor positioned before the first visit; ids are 1, 2, 3, ... and never reused.
    fn start(&self, strategy: &str) -> Result<u64, TraversalError>;

    /// The next value, or `Ok(None)` once all rows*cols values were returned. `UnknownCursor` for unknown/closed ids.
    fn next(&self, cursor: u64) -> Result<Option<i32>, TraversalError>;

    /// True while at least one value is left.
    fn has_next(&self, cursor: u64) -> Result<bool, TraversalError>;

    /// How many values the cursor has not returned yet.
    fn remaining(&self, cursor: u64) -> Result<u64, TraversalError>;

    /// Puts the cursor back before the first visit.
    fn reset(&self, cursor: u64) -> Result<(), TraversalError>;

    /// Releases the cursor; every later call with this id (a second close included) is `UnknownCursor`.
    fn close(&self, cursor: u64) -> Result<(), TraversalError>;

    /// Sum of the first `k` visited values; `k` above rows*cols counts as rows*cols. `k < 0`: `InvalidArgument`.
    fn path_sum(&self, strategy: &str, k: i64) -> Result<i64, TraversalError>;

    /// The value of the `k`-th visit, 0-based. `k < 0` or `k >= rows*cols`: `InvalidArgument`.
    fn kth_value(&self, strategy: &str, k: i64) -> Result<i32, TraversalError>;
}

/// Builds the service over `source`. `Err(InvalidMatrix)` if it has zero rows or zero columns.
pub type Created = Result<Arc<dyn MatrixTraversal>, TraversalError>;

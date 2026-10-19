//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`); statements behave as if they ran one after the other.
//!
//! Supported SQL (keywords case-insensitive, names case-sensitive, optional trailing ';'):
//!
//! ```text
//! CREATE TABLE t (col INT|TEXT [PRIMARY KEY], ...)
//! INSERT INTO t VALUES (literal, ...)                      -- one row; literals: 42, -7, 'text', 'it''s'
//! SELECT * | item, ... FROM t [JOIN u ON t.a = u.b]
//!        [WHERE cond] [GROUP BY col] [ORDER BY col [ASC|DESC]] [LIMIT n]
//!      item := col | t.col | COUNT(*) | SUM(col) | MIN(col) | MAX(col)
//! UPDATE t SET col = literal | col = col + n | col = col - n [, ...] [WHERE cond]
//! DELETE FROM t [WHERE cond]
//!      cond := comparison | cond AND cond | cond OR cond | ( cond )     -- AND binds tighter than OR
//!      comparison := col (= | != | <> | < | <= | > | >=) literal
//! ```

/// Any syntax or semantic error. The statement that caused it changed nothing. Provided.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SqlError(pub String);

/// What a statement returns. Provided.
///
/// * `columns`: output column names (empty for statements that return no rows)
/// * `rows`: every value as text: integers in decimal, text as is
/// * `affected`: rows inserted / updated / deleted (0 for SELECT and CREATE TABLE)
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct QueryResult {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<String>>,
    pub affected: usize,
}

pub trait SqlEngine: Send + Sync {
    /// Runs one statement. `Err(SqlError)` for anything invalid.
    fn execute(&self, sql: &str) -> Result<QueryResult, SqlError>;
}

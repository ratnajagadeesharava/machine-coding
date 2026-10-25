//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! A cell address is a column letter A..Z followed by a row number 1..100 (no leading zeros), e.g. "B12".
//! Addresses are case-insensitive and are reported in upper case. Every method that takes an address returns
//! `SheetError::InvalidAddress` for an invalid one ("", "A0", "A101", "AA1", "A01", " A1", ...) and changes nothing.
//! All lists are sorted row by row (A1, B1, ..., Z1, A2, ...), without duplicates.

#[derive(Debug, PartialEq, Eq)]
pub enum SheetError {
    InvalidAddress,
}

pub trait FormulaSheet: Send + Sync {
    /// Stores `input` in the cell, replacing whatever was there. "" clears the cell, a number literal makes a
    /// number, input starting with '=' makes a formula, anything else is text. After this call every cell that
    /// directly or indirectly depends on the cell is up to date.
    fn set(&self, cell: &str, input: &str) -> Result<(), SheetError>;

    /// The display string of the cell ("" for an empty cell); see the README for the exact formats.
    fn get(&self, cell: &str) -> Result<String, SheetError>;

    /// The formula text with letters upper-cased and nothing else changed; "" if the cell is not a formula.
    fn formula_of(&self, cell: &str) -> Result<String, SheetError>;

    /// The valid cells the formula in this cell refers to directly (ranges expanded); empty if it has none.
    fn dependencies(&self, cell: &str) -> Result<Vec<String>, SheetError>;

    /// The cells whose formulas refer to this cell directly (also through a range).
    fn dependents(&self, cell: &str) -> Result<Vec<String>, SheetError>;

    /// All non-empty cells.
    fn used_cells(&self) -> Vec<String>;

    /// Number of formula evaluations performed so far, including the one made when a formula is set.
    fn recompute_count(&self) -> u64;
}

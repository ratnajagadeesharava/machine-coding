//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Addresses are "A1" style: one or two letters (A..ZZ, columns 1..=702) followed by a row number 1..=1_000_000
//! without leading zeros. Input is case-insensitive (normalised to upper case); surrounding whitespace is NOT allowed.
//! A malformed or out-of-range address, or any other invalid argument, returns `GridError::InvalidArgument` and leaves
//! the grid unchanged. A "range" is a top-left and a bottom-right address; the top-left must not be below or right of
//! the bottom-right. Columns are passed as letters ("C").

#[derive(Debug, PartialEq, Eq)]
pub enum GridError {
    /// bad address / column / range, bad number or style value, area over the limit, destination outside the grid
    InvalidArgument,
}

/// What a cell currently holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueType {
    Empty,
    Number,
    Text,
}

/// A partial style update. A `None` field means "leave that part of the style as it is".
/// Valid values: `font_size` 8..=72; `text_color` and `fill_color` a non-empty name without whitespace and without `'|'`
/// (stored in lower case); `number_format` one of GENERAL, INTEGER, TWO_DECIMALS, PERCENT (case-insensitive, stored in
/// upper case).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StyleChange {
    pub bold: Option<bool>,
    pub italic: Option<bool>,
    pub font_size: Option<i32>,
    pub text_color: Option<String>,
    pub fill_color: Option<String>,
    pub number_format: Option<String>,
}

pub trait SpreadsheetGrid: Send + Sync {
    // ---- cells ----

    /// Stores a number (NaN and infinities are rejected). The cell's style is kept.
    fn set_number(&self, cell: &str, value: f64) -> Result<(), GridError>;

    /// Stores text. Empty text empties the cell's value but keeps its style.
    fn set_text(&self, cell: &str, text: &str) -> Result<(), GridError>;

    /// Removes the value AND the style of the cell: it becomes an unused default cell.
    fn clear(&self, cell: &str) -> Result<(), GridError>;

    /// Display string: "" for an empty cell, the text as is, or the number rendered by its number format.
    fn get_value(&self, cell: &str) -> Result<String, GridError>;

    fn value_type(&self, cell: &str) -> Result<ValueType, GridError>;

    // ---- styles ----

    /// Applies the `Some` fields of `change` to the cell's style (all or nothing).
    fn set_style(&self, cell: &str, change: &StyleChange) -> Result<(), GridError>;

    /// `bold|italic|fontSize|textColor|fillColor|numberFormat`, e.g. `true|false|11|black|none|GENERAL`.
    fn style_of(&self, cell: &str) -> Result<String, GridError>;

    /// Applies `change` to every cell of the range (at most 1_000_000 cells).
    fn set_style_range(
        &self,
        top_left: &str,
        bottom_right: &str,
        change: &StyleChange,
    ) -> Result<(), GridError>;

    /// Number of different non-default styles currently held by at least one cell.
    fn distinct_style_count(&self) -> usize;

    // ---- range operations (area limit 1_000_000 cells unless stated) ----

    /// Sets the value of every cell of the range; styles are kept.
    fn fill_number(&self, top_left: &str, bottom_right: &str, value: f64) -> Result<(), GridError>;

    /// Like `fill_number` with text. Empty text empties the values.
    fn fill_text(&self, top_left: &str, bottom_right: &str, text: &str) -> Result<(), GridError>;

    /// Clears value and style of every cell of the range (no area limit).
    fn clear_range(&self, top_left: &str, bottom_right: &str) -> Result<(), GridError>;

    /// Copies values and styles of the source range to a same-sized area whose top-left is `dest_top_left`.
    /// Empty source cells clear the destination cell (value and style). The result is as if the source were first
    /// copied to a temporary buffer. A destination that leaves the grid is an error.
    fn copy(
        &self,
        src_top_left: &str,
        src_bottom_right: &str,
        dest_top_left: &str,
    ) -> Result<(), GridError>;

    /// Cut and paste: like `copy`, then every source cell that is not inside the destination area is cleared
    /// (value and style).
    fn move_range(
        &self,
        src_top_left: &str,
        src_bottom_right: &str,
        dest_top_left: &str,
    ) -> Result<(), GridError>;

    // ---- structure ----

    /// Shifts rows `row..` down by one, leaving an empty row; cells pushed past row 1_000_000 vanish.
    fn insert_row(&self, row: u32) -> Result<(), GridError>;

    /// Removes the row; the rows below move up by one.
    fn delete_row(&self, row: u32) -> Result<(), GridError>;

    /// Shifts columns `column..` right by one, leaving an empty column; cells pushed past ZZ vanish.
    fn insert_column(&self, column: &str) -> Result<(), GridError>;

    /// Removes the column; the columns to its right move left by one.
    fn delete_column(&self, column: &str) -> Result<(), GridError>;

    // ---- queries ----

    /// Bounding box of all cells that hold a value or a non-default style, e.g. "B2:D7"; "" if there are none.
    fn used_range(&self) -> String;

    /// Largest row number of a used cell (0 if none).
    fn row_count(&self) -> u32;

    /// Largest column number of a used cell (A=1 ... ZZ=702; 0 if none).
    fn column_count(&self) -> u32;

    // ---- sorting ----

    /// Stable sort of the rows of the range by the values in column `by_column` (which must lie inside the range).
    /// Whole rows of the range (values and styles) move. Order: numbers, then text (compared case-insensitively),
    /// then empty values. Descending reverses the order inside the numbers group and inside the text group only;
    /// equal keys keep their original order in both directions.
    fn sort_range(
        &self,
        top_left: &str,
        bottom_right: &str,
        by_column: &str,
        ascending: bool,
    ) -> Result<(), GridError>;
}

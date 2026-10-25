//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Positions are character indexes (the tests use ASCII text only). Every error leaves the document unchanged.

#[derive(Debug, PartialEq, Eq)]
pub enum WordError {
    /// a position, range or line outside the document
    OutOfRange,
    /// empty pattern, unknown/invalid format field, duplicate format field, unknown alignment name
    InvalidArgument,
}

pub trait WordProcessor: Send + Sync {
    /// Inserts `text` before character `pos` (0 <= pos <= length; `pos == length` appends).
    /// The new characters take the format of the character before `pos`; at position 0, or in an empty document,
    /// they take the default format ("Calibri", 11, not bold, not italic, not underlined, "black").
    /// Inserting an empty string is allowed and changes nothing. `pos > length` is `OutOfRange`.
    fn insert(&self, pos: usize, text: &str) -> Result<(), WordError>;

    /// Deletes the half-open range [from, to). Requires from <= to <= length (else `OutOfRange`);
    /// an empty range changes nothing.
    fn delete(&self, from: usize, to: usize) -> Result<(), WordError>;

    /// Changes the format of the characters in [from, to) (same range rules as `delete`, `OutOfRange`).
    /// `change` holds only the fields to change as (key, value) pairs; the other fields keep their values.
    /// Keys and values: "font" (non-blank, no '|'), "size" (integer 1..=1000), "bold" / "italic" / "underline"
    /// ("true" or "false"), "color" (non-blank, no '|'). An unknown key, a repeated key or an invalid value is
    /// `InvalidArgument`. An empty slice changes nothing. Nothing is applied if anything is invalid.
    fn apply_format(&self, from: usize, to: usize, change: &[(&str, &str)]) -> Result<(), WordError>;

    /// The format of the character at `pos` (pos < length, else `OutOfRange`) as "font|size|B|I|U|color",
    /// e.g. "Calibri|11|-|-|-|black" or "Arial|14|B|-|U|red".
    fn format_at(&self, pos: usize) -> Result<String, WordError>;

    /// The whole text.
    fn text(&self) -> String;

    /// Number of characters.
    fn length(&self) -> usize;

    /// Maximal runs of equal format, in document order, as "start-end:formatString" (end exclusive),
    /// e.g. "0-5:Calibri|11|-|-|-|black". Empty document: empty vector.
    fn runs(&self) -> Vec<String>;

    /// Start indexes (ascending) of every match, overlapping ones included. An empty pattern is `InvalidArgument`.
    fn find(&self, pattern: &str, case_sensitive: bool) -> Result<Vec<usize>, WordError>;

    /// Replaces the non-overlapping matches found scanning left to right in the original text and returns how many
    /// were replaced. Every character of a replacement takes the format of the first character of the match it
    /// replaced. An empty pattern is `InvalidArgument`.
    fn replace_all(&self, pattern: &str, replacement: &str, case_sensitive: bool) -> Result<usize, WordError>;

    /// Number of maximal sequences of non-whitespace characters (whitespace: space, tab, '\n', '\r').
    fn word_count(&self) -> usize;

    /// Number of '\n' characters plus one.
    fn line_count(&self) -> usize;

    /// Sets the alignment of a line (line < line_count, else `OutOfRange`); alignment is one of
    /// "LEFT", "CENTER", "RIGHT", "JUSTIFY" (else `InvalidArgument`).
    fn set_alignment(&self, line: usize, alignment: &str) -> Result<(), WordError>;

    /// The alignment of a line (line < line_count, else `OutOfRange`); "LEFT" unless set or inherited.
    fn alignment(&self, line: usize) -> Result<String, WordError>;

    /// Number of distinct formats currently used by at least one character (0 for an empty document).
    fn distinct_format_count(&self) -> usize;
}

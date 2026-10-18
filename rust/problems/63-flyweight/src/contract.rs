//! Provided: the error type.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MapError {
    /// Blank shape or colour.
    InvalidStyle,
    /// Latitude outside -90..=90 or longitude outside -180..=180.
    InvalidPosition,
}

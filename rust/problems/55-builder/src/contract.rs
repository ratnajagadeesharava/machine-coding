//! Provided: the error type reported by `build()`.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BuildError {
    /// No URL was set.
    MissingUrl,
    /// URL does not start with http:// or https:// plus something.
    InvalidUrl,
    /// Method is not one of GET, POST, PUT, PATCH, DELETE.
    InvalidMethod,
    /// Blank header or query name.
    InvalidName,
    /// Timeout outside 1..=60000 milliseconds.
    InvalidTimeout,
    /// A body together with GET or DELETE.
    BodyNotAllowed,
}

//! Provided: the error type.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShortenError {
    /// Not an `http://` / `https://` URL with something after the scheme.
    InvalidUrl,
}

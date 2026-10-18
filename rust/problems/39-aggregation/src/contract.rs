//! Provided: the error type for songs.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MusicError {
    InvalidTitle,
    InvalidDuration,
}

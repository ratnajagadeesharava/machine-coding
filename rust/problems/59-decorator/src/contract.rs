//! Provided: the component trait and the error type.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SinkError(pub String);

/// The component interface. A sink may fail with a `SinkError` when delivery fails.
pub trait MessageSink {
    fn send(&self, message: &str) -> Result<(), SinkError>;
}

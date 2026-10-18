//! Provided: the error type.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TicketError {
    /// The operation is not allowed in the current state; the text is
    /// `"cannot <operation> a ticket in state <STATE>"`, e.g. `"cannot resolve a ticket in state NEW"`.
    Illegal(String),
    /// A blank argument (checked before the state).
    InvalidArgument,
    /// The ticket was already reopened twice.
    ReopenLimit,
}

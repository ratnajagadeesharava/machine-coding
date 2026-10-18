//! Provided: the error type.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum GameError {
    /// Blank name, item or slot, or a history limit of 0.
    InvalidInput,
    /// Already carrying the maximum number of items.
    InventoryFull,
}

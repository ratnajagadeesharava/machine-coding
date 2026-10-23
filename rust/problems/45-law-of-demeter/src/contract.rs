//! Provided: the traits and error types.

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PayError {
    /// Zero amount.
    InvalidAmount,
    /// Not enough funds.
    InsufficientFunds,
}

pub trait Wallet {
    fn balance(&self) -> u64;
    /// `InvalidAmount` for 0, `InsufficientFunds` when the balance is too low (balance unchanged).
    fn debit(&mut self, amount: u64) -> Result<(), PayError>;
}

pub trait Customer {
    fn name(&self) -> &str;
    fn can_afford(&self, amount: u64) -> bool;
    /// `InsufficientFunds` when the customer cannot afford it (nothing debited).
    fn pay(&mut self, amount: u64) -> Result<(), PayError>;
    fn wallet(&self) -> &dyn Wallet;
}

pub trait Order {
    fn customer(&self) -> &dyn Customer;
    fn shipping_city(&self) -> &str;
    /// Value of the goods in cents.
    fn item_total(&self) -> u64;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Outcome {
    Approved,
    Declined,
}

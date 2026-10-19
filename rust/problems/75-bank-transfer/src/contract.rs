//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

#[derive(Debug, PartialEq, Eq)]
pub enum BankError {
    UnknownAccount,
    AccountExists,
    /// amount == 0, or from == to
    InvalidRequest,
}

pub trait Bank: Send + Sync {
    /// Creates an account. `Err(AccountExists)` if the name is taken.
    fn open(&self, account: &str, balance: u64) -> Result<(), BankError>;

    fn balance(&self, account: &str) -> Result<u64, BankError>;

    /// Moves money atomically: either both balances change or neither does. `Ok(false)` (nothing changes) when
    /// `from` has less than `amount`; a transfer of exactly the balance succeeds.
    /// `Err(UnknownAccount)` for a missing account, `Err(InvalidRequest)` for amount == 0 or from == to.
    fn transfer(&self, from: &str, to: &str, amount: u64) -> Result<bool, BankError>;

    /// Sum of all balances. Must never include a half-applied transfer (no money "in flight").
    fn total(&self) -> u64;
}

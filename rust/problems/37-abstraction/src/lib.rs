//! Abstraction — `Account` is a trait: four required methods are the "abstract" part, four provided methods are
//! the shared behaviour you write once.
#![allow(unused_variables)]

pub mod contract;

use contract::AccountError;

/// Holds the balance. Add whatever fields you need.
pub struct AccountState;

impl AccountState {
    pub fn new() -> AccountState {
        todo!()
    }

    pub fn balance(&self) -> i64 {
        todo!()
    }

    pub fn add(&mut self, delta: i64) {
        todo!()
    }
}

pub trait Account {
    // --- abstract part: decided by each account type ---
    fn state(&self) -> &AccountState;
    fn state_mut(&mut self) -> &mut AccountState;
    /// Fee charged by `month_end`.
    fn monthly_fee(&self) -> i64;
    /// Whether withdrawing `amount` right now is allowed.
    fn can_withdraw(&self, amount: i64) -> bool;

    // --- shared part: write once, here ---
    fn balance(&self) -> i64 {
        todo!()
    }

    /// `amount > 0` or `Err(InvalidAmount)`.
    fn deposit(&mut self, amount: i64) -> Result<(), AccountError> {
        todo!()
    }

    /// `amount > 0` or `Err(InvalidAmount)`; asks `can_withdraw`, `Err(Declined)` leaves the balance unchanged.
    fn withdraw(&mut self, amount: i64) -> Result<(), AccountError> {
        todo!()
    }

    /// Subtracts `monthly_fee()` (even if that overdraws).
    fn month_end(&mut self) {
        todo!()
    }
}

/// Withdrawal allowed only if the balance afterwards is >= `min_balance`. Fee 0.
pub struct SavingsAccount;

impl SavingsAccount {
    pub fn new(min_balance: i64) -> SavingsAccount {
        todo!()
    }
}

impl Account for SavingsAccount {
    fn state(&self) -> &AccountState {
        todo!()
    }
    fn state_mut(&mut self) -> &mut AccountState {
        todo!()
    }
    fn monthly_fee(&self) -> i64 {
        todo!()
    }
    fn can_withdraw(&self, amount: i64) -> bool {
        todo!()
    }
}

/// Allowed if the balance afterwards is >= -overdraft_limit. Fee 200, waived while the balance is >= 100000.
pub struct CheckingAccount;

impl CheckingAccount {
    pub fn new(overdraft_limit: i64) -> CheckingAccount {
        todo!()
    }
}

impl Account for CheckingAccount {
    fn state(&self) -> &AccountState {
        todo!()
    }
    fn state_mut(&mut self) -> &mut AccountState {
        todo!()
    }
    fn monthly_fee(&self) -> i64 {
        todo!()
    }
    fn can_withdraw(&self, amount: i64) -> bool {
        todo!()
    }
}

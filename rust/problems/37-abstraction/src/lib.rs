//! Abstraction.
#![allow(unused_variables)]

pub mod contract;

use contract::AccountError;

/// Holds the balance.
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
    fn state(&self) -> &AccountState;
    fn state_mut(&mut self) -> &mut AccountState;
    fn monthly_fee(&self) -> i64;
    fn can_withdraw(&self, amount: i64) -> bool;

    fn balance(&self) -> i64 {
        todo!()
    }

    fn deposit(&mut self, amount: i64) -> Result<(), AccountError> {
        todo!()
    }

    fn withdraw(&mut self, amount: i64) -> Result<(), AccountError> {
        todo!()
    }

    fn month_end(&mut self) {
        todo!()
    }
}

/// Savings account.
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

/// Checking account.
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

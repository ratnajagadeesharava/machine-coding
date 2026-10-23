//! Checkout and shipping services.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// `new` never fails; `debit` follows the `Wallet` contract.
pub struct SimpleWallet;

impl SimpleWallet {
    pub fn new(initial: u64) -> SimpleWallet {
        todo!()
    }
}

impl Wallet for SimpleWallet {
    fn balance(&self) -> u64 {
        todo!()
    }
    fn debit(&mut self, amount: u64) -> Result<(), PayError> {
        todo!()
    }
}

/// A customer that pays out of its own wallet.
pub struct StandardCustomer;

impl StandardCustomer {
    pub fn new(name: &str, wallet: Box<dyn Wallet>) -> StandardCustomer {
        todo!()
    }
}

impl Customer for StandardCustomer {
    fn name(&self) -> &str {
        todo!()
    }
    fn can_afford(&self, amount: u64) -> bool {
        todo!()
    }
    fn pay(&mut self, amount: u64) -> Result<(), PayError> {
        todo!()
    }
    fn wallet(&self) -> &dyn Wallet {
        todo!()
    }
}

/// Takes payments.
pub struct Cashier;

impl Cashier {
    pub fn new() -> Cashier {
        todo!()
    }

    /// `Approved` after calling `customer.pay(amount)`; `Declined` (without calling `pay`) when
    /// `customer.can_afford(amount)` is false. `amount == 0` -> `Err(PayError::InvalidAmount)`.
    pub fn checkout(&mut self, customer: &mut dyn Customer, amount: u64) -> Result<Outcome, PayError> {
        todo!()
    }

    /// Sum of all approved amounts.
    pub fn revenue(&self) -> u64 {
        todo!()
    }
}

/// Prices shipping for an order.
pub struct ShippingCalculator;

impl ShippingCalculator {
    pub fn new() -> ShippingCalculator {
        todo!()
    }

    /// 0 when `order.item_total() >= 10000`. Otherwise by `order.shipping_city()` (case-insensitive):
    /// "paris" 1500, "lyon" 1800, any other city 2500.
    pub fn cost(&self, order: &dyn Order) -> u64 {
        todo!()
    }
}

//! Factory Method — the trait `CheckoutFlow` fixes the algorithm (`checkout`); implementors decide which gateway
//! to create through `create_gateway`. Trait default method = the "creator" base class.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// name() is "card". Accepts charges up to 100000 cents inclusive.
pub struct CardGateway;

impl PaymentGateway for CardGateway {
    fn name(&self) -> &str {
        todo!()
    }
    fn charge(&self, customer_id: &str, amount_cents: u64) -> bool {
        todo!()
    }
}

/// name() is "bank". Accepts charges of at least 1000 cents.
pub struct BankGateway;

impl PaymentGateway for BankGateway {
    fn name(&self) -> &str {
        todo!()
    }
    fn charge(&self, customer_id: &str, amount_cents: u64) -> bool {
        todo!()
    }
}

/// The creator. Implementors only supply the factory method; `checkout` is written once, here.
pub trait CheckoutFlow {
    /// The factory method: decides which concrete gateway to create.
    fn create_gateway(&self) -> Box<dyn PaymentGateway>;

    /// Blank customer -> `Err(InvalidCustomer)`, zero amount -> `Err(InvalidAmount)` (before `create_gateway` is
    /// called). Otherwise calls `create_gateway()` exactly once, charges, and returns
    /// `"RECEIPT <gateway name> <customer> <amount>"` or `"DECLINED <gateway name> <customer> <amount>"`.
    fn checkout(&self, customer_id: &str, amount_cents: u64) -> Result<String, CheckoutError> {
        todo!()
    }
}

/// Its factory method creates a `CardGateway`.
pub struct CardCheckout;

impl CheckoutFlow for CardCheckout {
    fn create_gateway(&self) -> Box<dyn PaymentGateway> {
        todo!()
    }
}

/// Its factory method creates a `BankGateway`.
pub struct BankTransferCheckout;

impl CheckoutFlow for BankTransferCheckout {
    fn create_gateway(&self) -> Box<dyn PaymentGateway> {
        todo!()
    }
}

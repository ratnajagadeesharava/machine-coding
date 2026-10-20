//! Factory Method.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// Card payment gateway.
pub struct CardGateway;

impl PaymentGateway for CardGateway {
    fn name(&self) -> &str {
        todo!()
    }
    fn charge(&self, customer_id: &str, amount_cents: u64) -> bool {
        todo!()
    }
}

/// Bank payment gateway.
pub struct BankGateway;

impl PaymentGateway for BankGateway {
    fn name(&self) -> &str {
        todo!()
    }
    fn charge(&self, customer_id: &str, amount_cents: u64) -> bool {
        todo!()
    }
}

/// Checkout algorithm shared by all payment channels.
pub trait CheckoutFlow {
    fn create_gateway(&self) -> Box<dyn PaymentGateway>;

    fn checkout(&self, customer_id: &str, amount_cents: u64) -> Result<String, CheckoutError> {
        todo!()
    }
}

/// Card checkout.
pub struct CardCheckout;

impl CheckoutFlow for CardCheckout {
    fn create_gateway(&self) -> Box<dyn PaymentGateway> {
        todo!()
    }
}

/// Bank transfer checkout.
pub struct BankTransferCheckout;

impl CheckoutFlow for BankTransferCheckout {
    fn create_gateway(&self) -> Box<dyn PaymentGateway> {
        todo!()
    }
}

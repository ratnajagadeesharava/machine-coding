//! Meal order checkout.
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// Prices an order.
pub struct OrderPricer;

impl OrderPricer {
    pub fn new() -> OrderPricer {
        todo!()
    }

    pub fn total(&self, items: &[Item], distance_m: u32, policy: &dyn DeliveryFeePolicy) -> Result<u64, OrderError> {
        todo!()
    }
}

/// Delivery fee by distance band.
pub struct DistanceBandFee;

impl DistanceBandFee {
    pub fn new() -> DistanceBandFee {
        todo!()
    }
}

impl DeliveryFeePolicy for DistanceBandFee {
    fn fee(&self, distance_m: u32) -> u64 {
        todo!()
    }
}

/// Card payment method.
pub struct CardPayment;

impl CardPayment {
    pub fn new(limit_cents: u64) -> CardPayment {
        todo!()
    }
}

impl PaymentMethod for CardPayment {
    fn charge(&self, amount_cents: u64) -> Result<PaymentResult, OrderError> {
        todo!()
    }
}

impl Refundable for CardPayment {
    fn refund(&self, reference: &str, amount_cents: u64) -> bool {
        todo!()
    }
}

/// Cash-on-delivery payment method.
pub struct CashOnDelivery;

impl CashOnDelivery {
    pub fn new(max_cash_cents: u64) -> CashOnDelivery {
        todo!()
    }
}

impl PaymentMethod for CashOnDelivery {
    fn charge(&self, amount_cents: u64) -> Result<PaymentResult, OrderError> {
        todo!()
    }
}

/// Places and refunds orders.
pub struct OrderProcessor;

impl OrderProcessor {
    pub fn new(pricer: OrderPricer, repository: Box<dyn OrderRepository>, notifier: Box<dyn Notifier>) -> OrderProcessor {
        todo!()
    }

    pub fn place(
        &self,
        order_id: &str,
        items: &[Item],
        distance_m: u32,
        fee: &dyn DeliveryFeePolicy,
        payment: &dyn PaymentMethod,
    ) -> Result<Placement, OrderError> {
        todo!()
    }

    pub fn refund(&self, order_id: &str, reference: &str, amount_cents: u64, method: &dyn Refundable) -> bool {
        todo!()
    }

    pub fn status(&self, order_id: &str) -> Option<String> {
        todo!()
    }
}

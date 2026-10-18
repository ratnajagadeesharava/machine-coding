//! SOLID in one mini-system: pricer (SRP), fee policies and payment methods (OCP), a payment contract (LSP),
//! a separate `Refundable` (ISP) and a processor that gets its collaborators injected (DIP).
#![allow(unused_variables)]

pub mod contract;

use contract::*;

/// SRP: does the pricing arithmetic and nothing else.
pub struct OrderPricer;

impl OrderPricer {
    pub fn new() -> OrderPricer {
        todo!()
    }

    /// Sum of quantity * unit_cents plus `policy.fee(distance_m)`. Empty items or a quantity of 0 -> `InvalidItems`.
    pub fn total(&self, items: &[Item], distance_m: u32, policy: &dyn DeliveryFeePolicy) -> Result<u64, OrderError> {
        todo!()
    }
}

/// 0 up to and including 1000 m; 300 up to and including 3000 m; beyond that 300 plus 100 for every started
/// kilometre past 3000 m (3001 -> 400, 4000 -> 400, 4001 -> 500).
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

/// Approves any charge up to `limit_cents` (inclusive) with references "card-1", "card-2", ... in the order of
/// approval; larger charges are declined. It can refund: a refund is accepted (true) when the reference was issued
/// by this card and the total refunded for that reference stays within the amount charged; otherwise false.
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

/// Approves any charge up to `max_cash_cents` (inclusive) with references "cod-1", "cod-2", ...; larger charges
/// are declined. Cash cannot be refunded through the system, so this type does NOT implement `Refundable`.
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

/// Orchestrates an order. It knows no concrete payment method, fee policy, database or channel.
pub struct OrderProcessor;

impl OrderProcessor {
    pub fn new(pricer: OrderPricer, repository: Box<dyn OrderRepository>, notifier: Box<dyn Notifier>) -> OrderProcessor {
        todo!()
    }

    /// Blank id -> `Err(InvalidId)`. total = pricer.total(...) (its error is returned). If total > 0: charge the
    /// payment method; approved -> status "PAID", declined -> "PAYMENT_DECLINED". If total == 0 nothing is charged
    /// (the method is not called), the status is "PAID" and the reference is "". Then, in this order:
    /// `repository.save(order_id, status, total)` and `notifier.notify(order_id, "Order {id} {status}")`.
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

    /// Asks `method` to refund. If it says true: `save(order_id, "REFUNDED", amount)`, then
    /// `notify(order_id, "Order {id} REFUNDED")`, and returns true. If false: nothing is saved or sent.
    /// Only `Refundable` is required, not a whole payment method.
    pub fn refund(&self, order_id: &str, reference: &str, amount_cents: u64, method: &dyn Refundable) -> bool {
        todo!()
    }

    pub fn status(&self, order_id: &str) -> Option<String> {
        todo!()
    }
}

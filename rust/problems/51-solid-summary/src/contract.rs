//! Provided: value types, the small abstractions and the error type. Amounts are in cents.

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Item {
    pub name: String,
    pub quantity: u32,
    pub unit_cents: u64,
}

/// `reference` is non-blank when approved and "" when declined.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PaymentResult {
    pub approved: bool,
    pub reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Placement {
    pub status: String,
    pub total_cents: u64,
    pub reference: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OrderError {
    /// Empty items, quantity 0.
    InvalidItems,
    /// Blank order id.
    InvalidId,
    /// A zero payment amount.
    InvalidAmount,
}

/// What Liskov substitution is checked against. EVERY payment method must:
/// - return `Err(OrderError::InvalidAmount)` for an amount of 0;
/// - for any amount > 0 NEVER fail: either approve or decline;
/// - approve with a non-blank, unique reference; decline with the reference "".
pub trait PaymentMethod {
    fn charge(&self, amount_cents: u64) -> Result<PaymentResult, OrderError>;
}

/// Something that can refund a charge.
pub trait Refundable {
    /// True if `amount_cents` was refunded against that charge reference.
    fn refund(&self, reference: &str, amount_cents: u64) -> bool;
}

/// How delivery is priced.
pub trait DeliveryFeePolicy {
    fn fee(&self, distance_m: u32) -> u64;
}

/// Where orders are kept.
pub trait OrderRepository {
    fn save(&self, order_id: &str, status: &str, total_cents: u64);
    fn status_of(&self, order_id: &str) -> Option<String>;
}

/// How the customer is told.
pub trait Notifier {
    fn notify(&self, order_id: &str, message: &str);
}

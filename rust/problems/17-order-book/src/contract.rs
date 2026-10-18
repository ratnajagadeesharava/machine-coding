//! Thin test entry point. Your domain model lives behind it.

pub trait OrderBookApi {
    /// `side` is "BUY" or "SELL". Returns the order id, or `None` if the request is invalid.
    fn submit(&mut self, side: &str, price: i64, quantity: i64) -> Option<u64>;

    /// True if the order was resting with quantity left and has been removed.
    fn cancel(&mut self, order_id: u64) -> bool;

    fn best_bid(&self) -> Option<i64>;

    fn best_ask(&self) -> Option<i64>;

    /// Total remaining quantity resting at exactly this price on that side.
    fn volume_at(&self, side: &str, price: i64) -> i64;

    /// Unfilled quantity; 0 if filled or cancelled; `None` if the id was never issued.
    fn remaining(&self, order_id: u64) -> Option<i64>;

    /// "buyId:sellId:price:quantity", oldest first.
    fn trades(&self) -> Vec<String>;
}

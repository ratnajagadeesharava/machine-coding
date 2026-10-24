//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`). All money is in whole cents.
//! A call that returns an error changes nothing.

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CartError {
    /// blank string, or a number outside the documented range
    InvalidArgument,
    /// a sku, cart id or coupon code that does not exist
    UnknownSku,
    UnknownCart,
    UnknownCoupon,
    /// duplicate sku, cart id or coupon code
    AlreadyExists,
    /// `remove_item` on a sku the cart has no line for
    NoSuchLine,
    /// not enough units on hand; carries the offending sku
    InsufficientStock(String),
    /// checkout of a cart without lines
    EmptyCart,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CouponKind {
    /// value 1..=100
    Percent,
    /// value >= 1 cent
    Fixed,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CouponStatus {
    None,
    Applied,
    NotEligible,
}

pub trait CartService: Send + Sync {
    // ---- catalogue ----

    /// `price_cents > 0`; non-blank sku/name/category; a second product with the same sku is `AlreadyExists`.
    fn add_product(
        &self,
        sku: &str,
        name: &str,
        price_cents: i64,
        category: &str,
        stock: u32,
    ) -> Result<(), CartError>;

    /// Replaces the units on hand.
    fn set_stock(&self, sku: &str, n: u32) -> Result<(), CartError>;

    /// Units currently on hand.
    fn stock(&self, sku: &str) -> Result<u32, CartError>;

    /// Tax rate of a category in basis points, 0..=10000. Categories without a rate are taxed 0.
    fn set_tax_rate(&self, category: &str, basis_points: u32) -> Result<(), CartError>;

    /// In every complete group of x+y units, y units are free. x >= 1, y >= 1. Replaces any promotion of the sku.
    fn set_buy_x_get_y(&self, sku: &str, x: u32, y: u32) -> Result<(), CartError>;

    /// Integer percent 1..=90 off the line subtotal. Replaces any promotion of the sku.
    fn set_percent_off(&self, sku: &str, percent: u32) -> Result<(), CartError>;

    /// Removes the promotion of the sku (no-op if there is none).
    fn clear_promotion(&self, sku: &str) -> Result<(), CartError>;

    // ---- coupons ----

    /// `min_subtotal_cents >= 0`, `max_uses >= 1`; a second coupon with the same code is `AlreadyExists`.
    fn define_coupon(
        &self,
        code: &str,
        kind: CouponKind,
        value: i64,
        min_subtotal_cents: i64,
        max_uses: u32,
    ) -> Result<(), CartError>;

    /// Successful checkouts that have consumed the coupon so far.
    fn coupon_uses(&self, code: &str) -> Result<u32, CartError>;

    // ---- carts ----

    /// A second cart with the same id is `AlreadyExists`.
    fn create_cart(&self, cart_id: &str, user_id: &str) -> Result<(), CartError>;

    /// Adds qty (>= 1) units to the sku's line. Beyond the units on hand: `InsufficientStock(sku)`, cart unchanged.
    fn add_item(&self, cart_id: &str, sku: &str, qty: u32) -> Result<(), CartError>;

    /// Sets the line to exactly qty units; 0 removes the line. Same stock rule as `add_item`.
    fn set_quantity(&self, cart_id: &str, sku: &str, qty: u32) -> Result<(), CartError>;

    /// Removes the sku's line; `NoSuchLine` if the cart has none.
    fn remove_item(&self, cart_id: &str, sku: &str) -> Result<(), CartError>;

    /// Units of the sku in the cart (0 if no line).
    fn quantity(&self, cart_id: &str, sku: &str) -> Result<u32, CartError>;

    /// Replaces the cart's coupon (the code must be defined). Eligibility is not checked here.
    fn apply_coupon(&self, cart_id: &str, code: &str) -> Result<(), CartError>;

    /// Detaches the cart's coupon (no-op if none).
    fn remove_coupon(&self, cart_id: &str) -> Result<(), CartError>;

    /// Evaluated now.
    fn coupon_status(&self, cart_id: &str) -> Result<CouponStatus, CartError>;

    /// "subtotal|promoDiscount|couponDiscount|tax|total" in cents, evaluated now.
    fn total(&self, cart_id: &str) -> Result<String, CartError>;

    /// Atomically: check stock of every line, decrement stock, consume the coupon use (if eligible), empty the cart
    /// (coupon included), record the order. Returns "ORD-1", "ORD-2", ... in checkout order.
    /// `InsufficientStock(sku)` names the first lacking sku in ascending sku order; `EmptyCart` for a cart without lines.
    fn checkout(&self, cart_id: &str) -> Result<String, CartError>;

    /// The user's orders in checkout order, each "orderId|totalCents". Empty for a user without orders.
    fn orders(&self, user_id: &str) -> Result<Vec<String>, CartError>;
}

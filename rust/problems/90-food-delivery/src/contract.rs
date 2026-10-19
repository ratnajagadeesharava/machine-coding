//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Everything lives on an integer grid; distance is Manhattan distance |x1-x2| + |y1-y2|.
//!
//! Order states: PLACED -> ACCEPTED -> READY -> PICKED_UP -> DELIVERED, or CANCELLED (from PLACED, ACCEPTED or READY).

#[derive(Debug, PartialEq, Eq)]
pub enum FoodError {
    /// unknown restaurant, item or order id
    UnknownId,
    /// empty order, quantity 0
    InvalidInput,
    OutOfStock,
    /// transition not allowed from the current state (or pickUp without a partner)
    InvalidState,
    /// cancel by somebody else than the customer
    NotAllowed,
}

pub type Food<T> = Result<T, FoodError>;

pub trait FoodDelivery: Send + Sync {
    fn add_restaurant(&self, id: &str, cuisine: &str, x: i32, y: i32);

    /// A menu item with `stock` portions available. Re-adding an item replaces its price and adds to the stock.
    fn add_item(&self, restaurant_id: &str, item: &str, price: u64, stock: u32) -> Food<()>;

    fn add_partner(&self, partner_id: &str, x: i32, y: i32);

    /// Restaurants of the cuisine within `max_distance` of (x, y), nearest first, ties by id.
    fn search_restaurants(&self, cuisine: &str, x: i32, y: i32, max_distance: u32) -> Vec<String>;

    /// Places an order for `item -> quantity` (all quantities >= 1) and reserves the stock, all-or-nothing.
    fn place_order(
        &self,
        customer: &str,
        restaurant_id: &str,
        items: &[(&str, u32)],
    ) -> Food<String>;

    fn total_of(&self, order_id: &str) -> Food<u64>;

    fn status_of(&self, order_id: &str) -> Food<String>;

    /// PLACED -> ACCEPTED.
    fn accept(&self, order_id: &str) -> Food<()>;

    /// ACCEPTED -> READY and the order looks for a delivery partner: the FREE partner nearest to the restaurant
    /// (ties by partner id) gets it. If nobody is free the order waits (first come, first served) and is assigned
    /// the moment a partner becomes free. A partner has at most one order at a time, from assignment until delivery
    /// or cancellation.
    fn mark_ready(&self, order_id: &str) -> Food<()>;

    /// The partner assigned to the order, if any.
    fn partner_of(&self, order_id: &str) -> Food<Option<String>>;

    /// READY (with a partner assigned) -> PICKED_UP.
    fn pick_up(&self, order_id: &str) -> Food<()>;

    /// PICKED_UP -> DELIVERED; the partner becomes free (and takes the oldest waiting order, if any).
    fn deliver(&self, order_id: &str) -> Food<()>;

    /// Cancels the order, restores its stock and frees its partner. Only the customer who placed it.
    fn cancel(&self, order_id: &str, customer: &str) -> Food<()>;

    /// Portions of the item still available for new orders.
    fn stock_of(&self, restaurant_id: &str, item: &str) -> Food<u32>;
}

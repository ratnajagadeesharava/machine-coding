//! Thin test entry point. Your domain model lives behind it.

pub trait VendingMachineApi {
    fn load_product(&mut self, slot: &str, name: &str, price_cents: u32, quantity: u32);

    /// Adds coins of the given denomination (5, 10, 25 or 100) to the machine's inventory.
    fn load_coins(&mut self, denomination: u32, count: u32);

    /// Returns false for unsupported denominations.
    fn insert_coin(&mut self, denomination: u32) -> bool;

    /// DISPENSED, INVALID_SLOT, SOLD_OUT, INSUFFICIENT_FUNDS or CANNOT_MAKE_CHANGE.
    fn select(&mut self, slot: &str) -> String;

    /// Refunds the inserted coins into the tray.
    fn cancel(&mut self);

    /// Credit currently inserted.
    fn balance(&self) -> u32;

    /// Empties the tray; coins in descending order.
    fn take_returned_coins(&mut self) -> Vec<u32>;

    fn stock_of(&self, slot: &str) -> u32;
}

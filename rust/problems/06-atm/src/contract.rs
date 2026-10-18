//! Thin test entry point. Your domain model lives behind it.

use std::collections::BTreeMap;

pub trait AtmApi {
    fn add_account(&mut self, card_id: &str, pin: &str, balance: u64);

    /// Adds notes of the given denomination (10, 20, 50 or 100) to the machine.
    fn load_cash(&mut self, denomination: u32, count: u32);

    /// Returns a session token, or `None` (unknown card, wrong PIN, locked card).
    fn authenticate(&mut self, card_id: &str, pin: &str) -> Option<String>;

    fn balance(&self, session: &str) -> Option<u64>;

    /// Notes dispensed (denomination -> count), or `None` if refused.
    fn withdraw(&mut self, session: &str, amount: u32) -> Option<BTreeMap<u32, u32>>;

    fn deposit(&mut self, session: &str, amount: u64) -> bool;

    fn end_session(&mut self, session: &str);

    fn cash_in_machine(&self) -> u64;
}

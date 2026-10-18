//! Thin test entry point. Your domain model lives behind it.

use std::collections::BTreeMap;

pub trait SplitwiseApi {
    fn add_equal_expense(&mut self, payer: &str, amount: i64, participants: &[&str]) -> bool;

    /// Shares must add up to `amount`.
    fn add_exact_expense(&mut self, payer: &str, amount: i64, shares: &[(&str, i64)]) -> bool;

    /// Percents (integers) must add up to 100.
    fn add_percent_expense(&mut self, payer: &str, amount: i64, percents: &[(&str, i64)]) -> bool;

    /// Net amount `debtor` owes `creditor` (>= 0).
    fn owed(&self, debtor: &str, creditor: &str) -> i64;

    /// Net position per person (positive = is owed); zero balances omitted.
    fn balances(&self) -> BTreeMap<String, i64>;

    fn settle(&mut self, from: &str, to: &str, amount: i64) -> bool;

    /// Payments "debtor>creditor:amount", sorted ascending, that would zero all balances.
    fn simplify(&self) -> Vec<String>;
}

//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Amounts are whole currency units. A failing call changes nothing.

use std::collections::BTreeMap;

#[derive(Debug, PartialEq, Eq)]
pub enum GroupError {
    UnknownGroup,
    UnknownMember,
    /// duplicate group name or duplicate member
    AlreadyExists,
    /// no members/participants, amount or share <= 0, duplicates in a list, paying yourself
    InvalidInput,
}

pub type Res<T> = Result<T, GroupError>;

pub trait ExpenseGroups: Send + Sync {
    /// Creates a group with at least one distinct member.
    fn create_group(&self, group: &str, members: &[&str]) -> Res<()>;

    fn add_member(&self, group: &str, member: &str) -> Res<()>;

    /// Removes a member only if their balance in the group is zero. Returns `Ok(false)` (and keeps them) otherwise.
    fn remove_member(&self, group: &str, member: &str) -> Res<bool>;

    /// `payer` paid `amount` for everyone in `participants` equally. A remainder is handed out one unit at a time
    /// to the participants in ascending name order. The payer need not be a participant.
    fn add_expense(&self, group: &str, payer: &str, amount: i64, participants: &[&str]) -> Res<()>;

    /// `payer` paid the sum of the shares; each member owes exactly their share. Shares must be > 0.
    fn add_exact_expense(&self, group: &str, payer: &str, shares: &[(&str, i64)]) -> Res<()>;

    /// `from` paid `to` `amount` to reduce what they owe. (Overpaying is allowed.)
    fn settle(&self, group: &str, from: &str, to: &str, amount: i64) -> Res<()>;

    /// Net position per member (positive = is owed money); zero balances omitted.
    fn balances(&self, group: &str) -> Res<BTreeMap<String, i64>>;

    /// Sum of all expenses (not settlements) of the group.
    fn total_spent(&self, group: &str) -> Res<i64>;

    /// A list of payments "from>to:amount" that brings every balance to zero using the FEWEST payments possible.
    /// Any optimal set is accepted; the list is sorted ascending. Empty if everybody is square.
    fn settle_up(&self, group: &str) -> Res<Vec<String>>;
}

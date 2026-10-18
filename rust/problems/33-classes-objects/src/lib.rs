//! Classes and Objects — implement `Money` here. Keep the public signatures: the tests compile against them.
#![allow(unused_variables)]

pub mod contract;

use contract::MoneyError;
use std::cmp::Ordering;
use std::fmt;

// Keep these derives: the tests use `==`, `HashSet`, `{:?}` and `.clone()`. Add your own fields.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Money;

impl Money {
    pub fn new(currency: &str, minor_units: i64) -> Result<Money, MoneyError> {
        todo!()
    }

    pub fn zero(currency: &str) -> Result<Money, MoneyError> {
        todo!()
    }

    pub fn currency(&self) -> &str {
        todo!()
    }

    pub fn minor_units(&self) -> i64 {
        todo!()
    }

    pub fn plus(&self, other: &Money) -> Result<Money, MoneyError> {
        todo!()
    }

    pub fn minus(&self, other: &Money) -> Result<Money, MoneyError> {
        todo!()
    }

    pub fn times(&self, factor: i32) -> Result<Money, MoneyError> {
        todo!()
    }

    pub fn is_negative(&self) -> bool {
        todo!()
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        todo!()
    }
}

/// Only values of the same currency are comparable; different currencies give `None`.
impl PartialOrd for Money {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        todo!()
    }
}

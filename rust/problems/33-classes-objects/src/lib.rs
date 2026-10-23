//! Classes and Objects.
#![allow(unused_variables)]

pub mod contract;

use contract::MoneyError;
use std::cmp::Ordering;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Money {
    currency: String,
    minor_units: i64,
}

impl Money {
    fn validate(currency: &str) -> Result<(), MoneyError> {
        if currency.len() != 3 {
            return Err(MoneyError::InvalidCurrency);
        }
        if currency.to_uppercase() != currency {
            return Err(MoneyError::InvalidCurrency);
        }
        Ok(())
    }
    pub fn new(currency: &str, minor_units: i64) -> Result<Money, MoneyError> {
        Money::validate(&currency)?;
        Ok(Money {
            currency: currency.to_string(),
            minor_units,
        })
    }

    pub fn zero(currency: &str) -> Result<Money, MoneyError> {
        Money::validate(&currency)?;
        Ok(Money {
            currency: currency.to_string(),
            minor_units: 0,
        })
    }

    pub fn currency(&self) -> &str {
        self.currency.as_str()
    }

    pub fn minor_units(&self) -> i64 {
        self.minor_units
    }

    pub fn plus(&self, other: &Money) -> Result<Money, MoneyError> {
        if (self.currency != other.currency) {
            Err(MoneyError::CurrencyMismatch)
        } else {
            let total = self
                .minor_units
                .checked_add(other.minor_units)
                .ok_or(MoneyError::Overflow)?;
            Ok(Money {
                currency: self.currency.clone(),
                minor_units: self.minor_units + other.minor_units,
            })
        }
    }

    pub fn minus(&self, other: &Money) -> Result<Money, MoneyError> {
        if (self.currency != other.currency) {
            Err(MoneyError::CurrencyMismatch)
        } else {
            Ok(Money {
                currency: self.currency.clone(),
                minor_units: self.minor_units - other.minor_units,
            })
        }
    }

    pub fn times(&self, factor: i32) -> Result<Money, MoneyError> {
        let total = self.minor_units.checked_mul(factor as i64).ok_or(MoneyError::Overflow)?;
        Ok(Money {
            currency: self.currency.clone(),
            minor_units: total,
        })
    }

    pub fn is_negative(&self) -> bool {
        self.minor_units < 0
    }
}

impl fmt::Display for Money {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let total = self.minor_units / 100;
        let decimal = (self.minor_units % 100).unsigned_abs();
        let sign = if self.minor_units < 0 { "-" } else { "" };
        let zero = if decimal < 10 { "0" } else { "" };
        let formated_currency = format!("{}{}.{}{}", sign, total, zero, decimal);
        write!(f, "{} {}", self.currency, formated_currency)
    }
}

/// Only values of the same currency are comparable; different currencies give `None`.
impl PartialOrd for Money {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        if self.currency != other.currency {
            None
        } else {
            self.minor_units.partial_cmp(&other.minor_units)
        }
    }
}

# 33 · Classes and Objects (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 30 min | Level 2 — a `Money` stub and `MoneyError` are provided; you write the bodies. |

## Problem

A shop's pricing code keeps passing around `f64` prices and loses cents. Build a `Money` **value type**:
an amount in the smallest currency unit (cents) plus a currency code.

## Requirements

1. `Money::new(currency, minor_units)` creates a value. The currency is exactly 3 upper-case ASCII letters; otherwise `Err(MoneyError::InvalidCurrency)`. `Money::zero(currency)` is shorthand.
2. `plus`, `minus` and `times` return a **new** value and borrow the operands (`&self`, `&Money`).
3. `plus` / `minus` between different currencies return `Err(MoneyError::CurrencyMismatch)`. `partial_cmp` between different currencies returns `None`.
4. Equal currency + amount means `==` and the same hash (it must work in a `HashSet`).
5. `PartialOrd` orders by amount.
6. `Display` prints `"USD 12.50"`; negative amounts print `"USD -0.05"`.
7. Overflow returns `Err(MoneyError::Overflow)` (no panics, no wrapping).
8. `is_negative()` is true for amounts below zero.

## Examples

```
Money::new("USD", 1250)?.to_string()          -> "USD 12.50"
usd(100).plus(&eur(100))                       -> Err(MoneyError::CurrencyMismatch)
```

## Running

```bash
cargo test -p p33_classes_objects     # from rust/ — tests FAIL until you implement Money
cargo play run 33
```

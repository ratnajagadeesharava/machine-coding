# 33 · Classes and Objects (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 30 min | Level 2 — a `Money` stub and `MoneyError` are provided; you write the bodies. |

## Problem

**What you are building.** A shop's pricing code keeps passing around `f64` prices and quietly loses cents
(`0.1 + 0.2 != 0.3`). You build a `Money` **value type**: an immutable amount stored as a whole number of the
smallest currency unit (cents) together with a 3-letter currency code. Two `Money` values that mean the same thing
must be interchangeable, so they can live in sets and be sorted, and mixing currencies or overflowing must fail
loudly (an `Err`) instead of producing a wrong price.

### Terms used below

- **Minor units**: the smallest unit of a currency; `1250` minor units of `USD` is 12 dollars 50 cents. Always 2 decimal digits here.
- **Value type**: a type defined entirely by its contents: two values with equal contents are interchangeable.
- **Major units**: the part before the decimal point when printed (`12` in `USD 12.50`).

## Requirements

1. `Money::new(currency, minor_units)` creates a value (`minor_units` is an `i64`, may be negative). The currency is exactly 3 upper-case ASCII letters; anything else — lower case, wrong length, empty — is `Err(MoneyError::InvalidCurrency)`. `Money::zero(currency)` is shorthand for zero minor units (same currency rule). `currency()` and `minor_units()` return what was given.
2. `plus`, `minus` and `times(i32 factor)` return a **new** value and borrow the operands (`&self`, `&Money`); the operands are never changed. `times` multiplies the amount by the (possibly negative) factor.
3. `plus` / `minus` between different currencies return `Err(MoneyError::CurrencyMismatch)`. `partial_cmp` between different currencies returns `None`.
4. Equal currency + amount means `==` and the same hash (it must work in a `HashSet`). Different amount or different currency means not equal.
5. `PartialOrd` orders by amount (negative < zero < positive); equal amounts compare as equal.
6. `Display` prints `"USD 12.50"`: currency, one space, major units, `.`, exactly two minor digits (`EUR 0.05`, `GBP 0.00`, `USD 1000.00`). Negative amounts print as `"USD -0.05"`: the minus sign goes right after the space, before the major units.
7. Overflow of `i64` in `plus`, `minus` or `times` returns `Err(MoneyError::Overflow)` (no panics, no wrapping).
8. `minus` may produce a negative amount; `is_negative()` is true exactly when the amount is below zero.

## Examples

```
Money::new("USD", 1250)?.to_string()          -> "USD 12.50"
Money::new("EUR", 5)?.to_string()             -> "EUR 0.05"
Money::new("USD", -5)?.to_string()            -> "USD -0.05"
usd(100).plus(&eur(100))                       -> Err(MoneyError::CurrencyMismatch)
usd(100).partial_cmp(&eur(100))                -> None
```

Arithmetic (a = USD 1000, b = USD 250; neither operand changes):

```
a.plus(&b)   -> Ok(USD 1250)    a.minus(&b) -> Ok(USD 750)     b.minus(&a) -> Ok(USD -750), is_negative() true
a.times(3)   -> Ok(USD 3000)    a.is_negative() -> false
usd(i64::MAX).plus(&usd(1))  -> Err(MoneyError::Overflow)
usd(i64::MAX).times(2)       -> Err(MoneyError::Overflow)
```

Sorting `[USD 300, USD -10, USD 120]` (with `partial_cmp`) gives `[USD -10, USD 120, USD 300]`.

## Edge cases

- `Money::new("usd", 1)`, `Money::new("US", 1)`, `Money::new("", 1)` and `Money::zero("DOLLARS")` are all `Err(MoneyError::InvalidCurrency)`.
- `USD 5` and `EUR 5` are not equal and not comparable.
- Zero prints with two decimals (`GBP 0.00`); amounts of 1000 dollars or more are printed without thousands separators.

## Running

```bash
cargo test -p p33_classes_objects     # from rust/ — tests FAIL until you implement Money
cargo play run 33
```

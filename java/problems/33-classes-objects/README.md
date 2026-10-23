# 33 · Classes and Objects (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 30 min | Level 2 — a `Money` stub with the signatures is provided; you write the bodies. |

## Problem

**What you are building.** A shop's pricing code keeps passing around `double` prices and quietly loses cents
(`0.1 + 0.2 != 0.3`). You build a `Money` **value object**: an immutable amount stored as a whole number of the
smallest currency unit (cents) together with a 3-letter currency code. Two `Money` values that mean the same thing
must be interchangeable, so they can live in sets and be sorted, and mixing currencies or overflowing must fail
loudly instead of producing a wrong price.

### Terms used below

- **Minor units**: the smallest unit of a currency; `1250` minor units of `USD` is 12 dollars 50 cents. Always 2 decimal digits here.
- **Value object**: an object defined entirely by its contents, not by identity: two instances with equal contents are interchangeable.
- **Major units**: the part before the decimal point when printed (`12` in `USD 12.50`).

## Requirements

1. `Money.of(currency, minorUnits)` creates a value (`minorUnits` is a `long`, may be negative). The currency is exactly 3 upper-case letters (`"USD"`); anything else — lower case, wrong length, `null`, empty — is an `IllegalArgumentException`. `Money.zero(currency)` is shorthand for zero minor units (same currency rule). `currency()` and `minorUnits()` return what was given.
2. `plus`, `minus` and `times(int factor)` return a **new** object and never change the operands. `times` multiplies the amount by the (possibly negative) factor.
3. `plus` / `minus` / `compareTo` only work for the same currency; otherwise throw `IllegalArgumentException`.
4. Two `Money` objects with the same currency and amount are `equals` and have the same `hashCode`, even though they are different instances. Different amount or different currency means not equal. They must work as `HashSet` members.
5. `compareTo` orders by amount (negative < zero < positive), so a list of `Money` can be sorted; equal amounts compare as `0`.
6. `toString()` prints `"USD 12.50"`: currency, one space, major units, `.`, exactly two minor digits (`EUR 0.05`, `GBP 0.00`, `USD 1000.00`). Negative amounts print as `"USD -0.05"`: the minus sign goes right after the space, before the major units.
7. Arithmetic that overflows `long` (`plus`, `minus`, `times`) throws `ArithmeticException` (never wraps silently).
8. `minus` may produce a negative amount; `isNegative()` is `true` exactly when the amount is below zero.

## Examples

```
Money.of("USD", 1250).toString()                 -> "USD 12.50"
Money.of("EUR", 5).toString()                    -> "EUR 0.05"
Money.of("USD", -5).toString()                   -> "USD -0.05"
Money.of("USD", 100).plus(Money.of("EUR", 100))  -> IllegalArgumentException
Money.of("USD", 5) == Money.of("USD", 5)         -> false   (two objects)
Money.of("USD", 5).equals(Money.of("USD", 5))    -> true    (same value)
```

Arithmetic (a = USD 1000, b = USD 250; neither operand changes):

```
a.plus(b)   -> USD 1250      a.minus(b) -> USD 750       b.minus(a) -> USD -750 (isNegative() true)
a.times(3)  -> USD 3000      a.isNegative() -> false
Money.of("USD", Long.MAX_VALUE).plus(Money.of("USD", 1))  -> ArithmeticException
Money.of("USD", Long.MAX_VALUE).times(2)                  -> ArithmeticException
```

Sorting `[USD 300, USD -10, USD 120]` gives `[USD -10, USD 120, USD 300]`.

## Edge cases

- `Money.of("usd", 1)`, `Money.of("US", 1)`, `Money.of(null, 1)` and `Money.zero("DOLLARS")` are all `IllegalArgumentException`.
- `Money.of("USD", 5)` and `Money.of("EUR", 5)` are not equal; comparing them with `compareTo` throws.
- Zero prints with two decimals (`GBP 0.00`); amounts of 1000 dollars or more are printed without thousands separators.

## Running

```bash
mvn test                 # from this directory — tests FAIL until you implement Money
```

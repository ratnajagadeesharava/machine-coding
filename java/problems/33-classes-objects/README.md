# 33 · Classes and Objects (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 30 min | Level 2 — a stub class with the signatures the tests compile against. Bodies throw `UnsupportedOperationException`. |

> **Concept:** a class is a blueprint, an object is one instance of it. Practise state, factories,
> immutability, and the difference between *identity* (`==`) and *equality* (`equals`).

## Problem

A shop's pricing code keeps passing around `double` prices and loses cents. Build a `Money` **value object**:
an amount in the smallest currency unit (cents) plus a currency code.

## Requirements

1. `Money.of(currency, minorUnits)` creates a value. The currency is exactly 3 upper-case letters (`"USD"`); anything else is an `IllegalArgumentException`. `Money.zero(currency)` is shorthand for zero.
2. `Money` is **immutable**: `plus`, `minus` and `times` return a **new** object and never change the operands.
3. `plus` / `minus` / `compareTo` only work for the same currency; otherwise throw `IllegalArgumentException`.
4. Two `Money` objects with the same currency and amount are `equals` and have the same `hashCode`, even though they are different instances. They must work as `HashSet` members.
5. `compareTo` orders by amount, so a list of `Money` can be sorted.
6. `toString()` prints `"USD 12.50"`: currency, space, major units, `.`, two minor digits. Negative amounts print as `"USD -0.05"`.
7. Arithmetic that overflows `long` throws `ArithmeticException` (never wraps silently).
8. `minus` may produce a negative amount (`isNegative()` is then `true`).

## Constraints

- No `double`/`float` anywhere. One currency per object, no exchange rates.

## Examples

```
Money.of("USD", 1250).toString()                 -> "USD 12.50"
Money.of("USD", 100).plus(Money.of("EUR", 100))  -> IllegalArgumentException
Money.of("USD", 5) == Money.of("USD", 5)         -> false   (two objects)
Money.of("USD", 5).equals(Money.of("USD", 5))    -> true    (same value)
```

## Behavioral contract

Anything not listed above is yours to decide. `src/main/java/solution/Money.java` is the stub you edit; keep its public signatures.

## Running

```bash
mvn test                 # from this directory — tests FAIL until you implement Money
```

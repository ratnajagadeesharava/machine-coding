# 48 · LSP — Liskov Substitution Principle (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — the `Coupon` contract is provided; the rest is yours to write. |

## Problem

A web shop has several kinds of coupons. `Checkout` should work with any of them without knowing which kind it is holding.

## Requirements

Read the contract in `contract/Coupon.java` carefully — it is the specification every coupon must meet.

1. `PercentCoupon(code, percent)`: percent `1..100` (else `IllegalArgumentException`); discount = `amount × percent / 100` rounded **down**.
2. `FlatCoupon(code, offCents)`: `offCents >= 0`; price = `max(0, amount − off)`.
3. `MinSpendCoupon(code, minSpend, off)`: `amount >= minSpend` → `max(0, amount − off)`; below the minimum → the amount unchanged (**no exception**).
4. All coupons: negative amount → `IllegalArgumentException` (exactly that type).
5. `Checkout.total(prices, coupon)`: sum the prices, apply the coupon **once** to the sum; `null` coupon = no discount; `null`/empty list = 0; negative price → `IllegalArgumentException`.

## How it is tested

`CouponContractTest` is one shared suite. It is run against `PercentCoupon`, `FlatCoupon`, `MinSpendCoupon` **and
a `Stacked` coupon defined in the test** (`StackedCouponTest`, built from your `PercentCoupon` and `FlatCoupon`). The same six contract tests must pass for each.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

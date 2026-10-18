# 48 · LSP — Liskov Substitution Principle (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — the `Coupon` trait, its documented contract and `CouponError` are provided; three coupons and `checkout_total` are stubs. |

> **Principle:** anything that accepts a `&dyn Coupon` must work with *every* `Coupon`. An implementor must keep the
> promises of the trait: accept at least what the trait accepts, and return at least what it promises.

## Problem

A web shop has several kinds of coupons. Checkout should not know which kind it is holding. Two classic mistakes
break that: a flat coupon larger than the price that produces a **negative** total, and a minimum-spend coupon that
returns an **error** when the basket is too small instead of simply not applying.

## Requirements

Read the contract at the top of `src/contract.rs` carefully — it is the specification every coupon must meet.

1. `PercentCoupon::new(code, percent)`: percent `1..=100` (else `Err(InvalidArgument)`); discount = `amount × percent / 100` rounded **down**.
2. `FlatCoupon::new(code, off)`: `off >= 0`; price = `max(0, amount − off)`.
3. `MinSpendCoupon::new(code, min_spend, off)`: `amount >= min_spend` → `max(0, amount − off)`; below the minimum → the amount unchanged (**not** an error).
4. All coupons: negative amount → `Err(CouponError::NegativeAmount)`.
5. `checkout_total(prices, coupon)`: sum, apply the coupon **once** to the sum; `None` = no discount; empty = 0; negative price → `Err(NegativeAmount)`.

## How it is tested

`contract_suite(&dyn Coupon)` in the test file is one shared set of checks. It is run against `PercentCoupon`,
`FlatCoupon`, `MinSpendCoupon` **and a `Stacked` coupon defined in the test** (built from your `PercentCoupon` and `FlatCoupon`).

## Running

```bash
cargo test -p p48_lsp     # from rust/ — tests FAIL until you implement the stubs
cargo play run 48
```

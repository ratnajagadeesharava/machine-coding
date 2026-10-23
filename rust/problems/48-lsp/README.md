# 48 · LSP — Liskov Substitution Principle (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — the `Coupon` trait, its documented contract and `CouponError` are provided; the rest is yours to write. |

## Problem

A web shop has several kinds of coupons. Checkout should work with any of them without knowing which kind it is holding.

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

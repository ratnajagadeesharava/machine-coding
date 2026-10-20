# 53 · Factory Method (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — the `PaymentGateway` trait and `CheckoutError` are provided; the rest is yours to write. |

## Problem

An online shop has one checkout algorithm (validate → create a gateway → charge → produce a receipt) but several
payment channels. Each channel has its own gateway.

## Requirements

1. `CardGateway` — `name()` is `"card"`; accepts charges up to **100000** cents inclusive. `BankGateway` — `name()` is `"bank"`; accepts charges of at least **1000** cents.
2. `CheckoutFlow::checkout(customer_id, amount)` (a provided method of the trait):
   - blank customer → `Err(CheckoutError::InvalidCustomer)`, `amount == 0` → `Err(InvalidAmount)`, **before** `create_gateway()` is called;
   - otherwise the gateway is created via `create_gateway()` (once per checkout), the charge is made, and the result is `"RECEIPT <gateway name> <customer> <amount>"` or `"DECLINED <gateway name> <customer> <amount>"`.
3. `CardCheckout` and `BankTransferCheckout` return the matching gateway from `create_gateway`.

## Running

```bash
cargo test -p p53_factory_method     # from rust/ — tests FAIL until you implement the stubs
cargo play run 53
```

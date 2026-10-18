# 53 · Factory Method (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — the `PaymentGateway` trait and `CheckoutError` are provided; two gateways, the `CheckoutFlow` trait and two creators are stubs. |

> **Pattern:** *Factory Method* — define an operation for creating an object, but let the implementor decide which
> type to create. The fixed algorithm lives in the trait's **default method**; it calls the required factory
> method and never names a concrete product.
>
> **Rust adaptation:** the "abstract creator class" is a trait: `create_gateway` is the required (abstract) method,
> `checkout` is the provided method that calls it.

## Problem

An online shop has one checkout algorithm (validate → create a gateway → charge → produce a receipt) but several
payment channels. Adding a channel must mean adding a type that implements `create_gateway`, not copying the
checkout code or adding a `match` on a channel enum.

## Requirements

1. `CardGateway` — `name()` is `"card"`; accepts charges up to **100000** cents inclusive. `BankGateway` — `name()` is `"bank"`; accepts charges of at least **1000** cents.
2. `CheckoutFlow::checkout(customer_id, amount)` (a provided method you write once in the trait):
   - blank customer → `Err(CheckoutError::InvalidCustomer)`, `amount == 0` → `Err(InvalidAmount)`, **before** `create_gateway()` is called;
   - otherwise `create_gateway()` is called **exactly once**, the charge is made, and the result is `"RECEIPT <gateway name> <customer> <amount>"` or `"DECLINED <gateway name> <customer> <amount>"`.
3. `CardCheckout` and `BankTransferCheckout` implement only `create_gateway`, returning the matching gateway.

## Constraints (what the tests catch)

- The tests implement `CheckoutFlow` themselves (with a recording gateway) and expect the default `checkout` to work for it.

## Running

```bash
cargo test -p p53_factory_method     # from rust/ — tests FAIL until you implement the stubs
cargo play run 53
```

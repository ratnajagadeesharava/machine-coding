# 53 · Factory Method (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — the `PaymentGateway` interface is provided; the rest is yours to write. |

## Problem

An online shop has one checkout algorithm (validate → create a gateway → charge → produce a receipt) but several
payment channels. Each channel has its own gateway.

## Requirements

1. `CardGateway` — `name()` is `"card"`; accepts charges up to **100000** cents inclusive. `BankGateway` — `name()` is `"bank"`; accepts charges of at least **1000** cents.
2. `CheckoutFlow` is an abstract class with `protected abstract PaymentGateway createGateway()` and `public String checkout(customerId, amount)`:
   - blank customer or `amount <= 0` → `IllegalArgumentException`, **before** `createGateway()` is called;
   - otherwise the gateway is created via `createGateway()` (once per checkout), the charge is made, and the result is `"RECEIPT <gateway name> <customer> <amount>"` or `"DECLINED <gateway name> <customer> <amount>"`.
3. `CardCheckout` and `BankTransferCheckout` return the matching gateway from `createGateway()`.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

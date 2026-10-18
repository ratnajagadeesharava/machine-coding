# 53 · Factory Method (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — the `PaymentGateway` product interface is provided; two gateways, the abstract creator and two concrete creators are stubs. |

> **Pattern:** *Factory Method* — define an operation for creating an object, but let **subclasses decide which
> class to instantiate**. The base class contains the fixed algorithm and calls the abstract factory method; it
> never names a concrete product.

## Problem

An online shop has one checkout algorithm (validate → create a gateway → charge → produce a receipt) but several
payment channels. Adding a channel must mean adding a subclass that overrides `createGateway()`, not copying the
checkout code or adding an `if (type == ...)`.

## Requirements

1. `CardGateway` — `name()` is `"card"`; accepts charges up to **100000** cents inclusive. `BankGateway` — `name()` is `"bank"`; accepts charges of at least **1000** cents.
2. `CheckoutFlow` (abstract creator) has `protected abstract PaymentGateway createGateway()` and `public final String checkout(customerId, amount)`:
   - blank customer or `amount <= 0` → `IllegalArgumentException`, **before** `createGateway()` is called;
   - otherwise `createGateway()` is called **exactly once**, the charge is made, and the result is `"RECEIPT <gateway name> <customer> <amount>"` or `"DECLINED <gateway name> <customer> <amount>"`.
3. `CardCheckout` and `BankTransferCheckout` are the concrete creators whose factory methods return the matching gateway.

## Constraints (what the tests catch)

- The tests subclass `CheckoutFlow` themselves with a recording gateway. If `checkout` creates its own gateway, or switches on a type, they fail.
- Reflection: `CheckoutFlow` is `abstract`, `checkout` is `final`, `createGateway` is `protected abstract`.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

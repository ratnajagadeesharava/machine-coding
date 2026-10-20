# 53 · Factory Method (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — the `PaymentGateway` interface is provided; the rest is yours to write. |

## Problem

**What you are building.** An online shop has one checkout procedure — validate the request, obtain a payment gateway,
charge the customer, and produce a receipt line — but several payment channels (card, bank transfer, and others added
later). Each channel talks to its own gateway with its own rules. A new channel must be addable by supplying only its own
gateway and its own checkout type; the shared procedure itself is written once. Amounts are in **cents**.

**Glossary**
- *Gateway* — a `PaymentGateway`: `name()` (short label like `"card"`) and `charge(customerId, amountCents)` returning `true` if accepted.
- *Declined* — the gateway's `charge` returned `false`; this is a normal outcome (a string), not an exception.

## Requirements

1. `CardGateway` — `name()` is `"card"`; `charge` accepts amounts up to **100000** cents inclusive (100001 is declined). `BankGateway` — `name()` is `"bank"`; `charge` accepts amounts of at least **1000** cents (999 is declined).
2. `CheckoutFlow` is an abstract class with `protected abstract PaymentGateway createGateway()` and `public final String checkout(String customerId, long amountCents)`:
   - null or blank customer, or `amountCents <= 0` → `IllegalArgumentException`, **before** `createGateway()` is called;
   - otherwise the gateway is created via `createGateway()` (exactly once per `checkout` call, so three checkouts mean three creations), the charge is made with the customer and amount, and the result is `"RECEIPT <gateway name> <customer> <amount>"` if accepted or `"DECLINED <gateway name> <customer> <amount>"` if not. Fields are separated by single spaces; the amount is printed as a plain integer.
3. `CardCheckout` and `BankTransferCheckout` (no-arg constructors, subclasses of `CheckoutFlow`) return the matching gateway from `createGateway()`.
4. A subclass written by someone else (e.g. one in a test returning a gateway named `"fake"`) must work: whatever gateway it creates is the one used and named in the result.

## Examples

| Call | Result |
|---|---|
| `new CardCheckout().checkout("cust-1", 5000)` | `RECEIPT card cust-1 5000` |
| `new CardCheckout().checkout("cust-1", 100000)` | `RECEIPT card cust-1 100000` |
| `new CardCheckout().checkout("cust-1", 100001)` | `DECLINED card cust-1 100001` |
| `new BankTransferCheckout().checkout("cust-1", 250000)` | `RECEIPT bank cust-1 250000` |
| `new BankTransferCheckout().checkout("cust-1", 999)` | `DECLINED bank cust-1 999` |
| `new BankTransferCheckout().checkout("cust-1", 1000)` | `RECEIPT bank cust-1 1000` |
| custom flow whose gateway is named `"fake"` and accepts: `checkout("cust-7", 1234)` | `RECEIPT fake cust-7 1234` |
| any flow: `checkout(" ", 100)`, `checkout(null, 100)`, `checkout("c", 0)`, `checkout("c", -5)` | `IllegalArgumentException`, no gateway created |

## Edge cases

- Boundaries: card 100000 ok / 100001 declined; bank 999 declined / 1000 ok.
- A rejected request (bad customer or amount) must not create a gateway or charge anything.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

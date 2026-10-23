# 53 · Factory Method (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — the `PaymentGateway` trait and `CheckoutError` are provided; the rest is yours to write. |

## Problem

**What you are building.** An online shop has one checkout procedure — validate the request, obtain a payment gateway,
charge the customer, and produce a receipt line — but several payment channels (card, bank transfer, and others added
later). Each channel talks to its own gateway with its own rules. A new channel must be addable by supplying only its own
gateway and its own checkout type; the shared procedure itself is written once. Amounts are in **cents** (`u64`).

**Glossary**
- *Gateway* — a `PaymentGateway`: `name()` (short label like `"card"`) and `charge(customer_id, amount_cents)` returning `true` if accepted.
- *Declined* — the gateway's `charge` returned `false`; this is a normal outcome (an `Ok` string), not an error.

## Requirements

1. `CardGateway` — `name()` is `"card"`; `charge` accepts amounts up to **100000** cents inclusive (100001 is declined). `BankGateway` — `name()` is `"bank"`; `charge` accepts amounts of at least **1000** cents (999 is declined).
2. `CheckoutFlow::checkout(&self, customer_id, amount_cents) -> Result<String, CheckoutError>` (a provided method of the `CheckoutFlow` trait, which also has the required method `create_gateway(&self) -> Box<dyn PaymentGateway>`):
   - blank (empty/whitespace) customer → `Err(CheckoutError::InvalidCustomer)`; `amount_cents == 0` → `Err(CheckoutError::InvalidAmount)`; both **before** `create_gateway()` is called;
   - otherwise the gateway is created via `create_gateway()` (exactly once per `checkout` call, so three checkouts mean three creations), the charge is made with the customer and amount, and the result is `Ok("RECEIPT <gateway name> <customer> <amount>")` if accepted or `Ok("DECLINED <gateway name> <customer> <amount>")` if not. Fields are separated by single spaces; the amount is printed as a plain integer.
3. `CardCheckout` and `BankTransferCheckout` (unit structs) return the matching gateway from `create_gateway`.
4. An implementor written by someone else (e.g. one in a test returning a gateway named `"fake"`) must work, including when held as `Box<dyn CheckoutFlow>`: whatever gateway it creates is the one used and named in the result.

## Examples

| Call | Result |
|---|---|
| `CardCheckout.checkout("cust-1", 5000)` | `Ok("RECEIPT card cust-1 5000")` |
| `CardCheckout.checkout("cust-1", 100000)` | `Ok("RECEIPT card cust-1 100000")` |
| `CardCheckout.checkout("cust-1", 100001)` | `Ok("DECLINED card cust-1 100001")` |
| `BankTransferCheckout.checkout("cust-1", 250000)` | `Ok("RECEIPT bank cust-1 250000")` |
| `BankTransferCheckout.checkout("cust-1", 999)` | `Ok("DECLINED bank cust-1 999")` |
| `BankTransferCheckout.checkout("cust-1", 1000)` | `Ok("RECEIPT bank cust-1 1000")` |
| custom flow whose gateway is named `"fake"` and accepts: `checkout("cust-7", 1234)` | `Ok("RECEIPT fake cust-7 1234")` |
| any flow: `checkout(" ", 100)` / `checkout("c", 0)` | `Err(InvalidCustomer)` / `Err(InvalidAmount)`, no gateway created |

## Edge cases

- Boundaries: card 100000 ok / 100001 declined; bank 999 declined / 1000 ok.
- A rejected request (bad customer or amount) must not create a gateway or charge anything.

## Running

```bash
cargo test -p p53_factory_method     # from rust/ — tests FAIL until you implement the stubs
cargo play run 53
```

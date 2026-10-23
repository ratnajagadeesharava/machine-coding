# 51 · SOLID Summary — Meal Order Checkout (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 75 min | Level 2 — value types, six small traits and `OrderError` are provided in `src/contract.rs`; the rest is yours to write in `src/lib.rs`. |

## Problem

**What you are building.** A food-delivery app takes an order (a list of meal items and how far away the customer is),
prices it (items plus a distance-based delivery fee), charges the customer through a payment method, records the outcome,
tells the customer, and can refund later. Payment methods, delivery-fee rules, the order store and the notifier are all
swappable: new ones must work with the checkout without the checkout changing. All money is in **cents** (`u64`);
distances are in **metres** (`u32`).

**Glossary**
- *Item* — `Item { name, quantity: u32, unit_cents: u64 }`: one order line, worth `quantity × unit_cents`.
- *Fee policy* — a `DeliveryFeePolicy`: `fee(distance_m)` returns the delivery charge in cents.
- *Reference* — the id a payment method gives to an approved charge; it is later used to refund that charge.
- *Free order* — an order whose total is 0 cents.
- *Started kilometre* — any part of a kilometre counts as a whole one (3001 m is 1 m into the first kilometre past 3000, so it counts as one).

## Requirements

Provided in `src/contract.rs`: `Item`, `PaymentResult { approved, reference }`, `Placement { status, total_cents, reference }`, `OrderError { InvalidItems, InvalidId, InvalidAmount }`, and the traits `PaymentMethod`, `Refundable`, `DeliveryFeePolicy`, `OrderRepository`, `Notifier`. You write `OrderPricer`, `DistanceBandFee`, `CardPayment`, `CashOnDelivery`, `OrderProcessor` in `src/lib.rs`.

1. `OrderPricer::total(items, distance_m, policy) -> Result<u64, OrderError>` returns `Σ quantity × unit_cents + policy.fee(distance_m)`. Empty `items` or any item with quantity 0 → `Err(OrderError::InvalidItems)`. A unit price of 0 is allowed.
2. `DistanceBandFee::fee(distance_m)`: 0–1000 m → 0; 1001–3000 m → 300; beyond 3000 m: 300 + 100 per *started* kilometre past 3000 (3001→400, 4000→400, 4001→500, 5000→500, 5001→600).
3. `CardPayment::new(limit_cents)`: a charge up to the limit (inclusive) is approved with reference `card-1`, `card-2`, … (a counter that advances **only on approvals**); a larger charge is declined (`approved: false`, reference `""`) — a decline is `Ok`, not an error. `refund(reference, amount)` returns `true` only when the reference is one this card issued **and** the cumulative refunds for that reference (including this one) stay `<=` the amount originally charged; otherwise `false` and nothing is recorded. It also implements `Refundable`.
4. `CashOnDelivery::new(max_cash_cents)`: approves up to `max_cash_cents` inclusive (references `cod-1`, `cod-2`, …, advancing only on approvals), declines more. It does **not** implement `Refundable`.
5. Every `PaymentMethod` follows the contract in `src/contract.rs`: `charge(0)` → `Err(OrderError::InvalidAmount)`; any amount `> 0` (even `u64::MAX`) never fails — it returns `Ok` with an approval or a decline; approved ⇒ non-blank, unique reference; declined ⇒ reference `""`.
6. `OrderProcessor::new(OrderPricer, Box<dyn OrderRepository>, Box<dyn Notifier>)`. `place(order_id, items, distance_m, fee, payment) -> Result<Placement, OrderError>`:
   - blank (empty/whitespace) `order_id` → `Err(OrderError::InvalidId)`, before anything else happens (no charge, no save, no notify);
   - price the order with the pricer (its error propagates);
   - if total `> 0`, call `payment.charge(total)`; approved → status `"PAID"` with the payment's reference, declined → status `"PAYMENT_DECLINED"` with reference `""`;
   - a free order (total 0) is `"PAID"` with reference `""` and the payment method is **never called**;
   - then, in this order: `repository.save(order_id, status, total)`, then `notifier.notify(order_id, "Order {order_id} {status}")`;
   - returns `Placement { status, total_cents, reference }`.
7. `OrderProcessor::refund(order_id, reference, amount_cents, &dyn Refundable) -> bool`: call `method.refund(reference, amount_cents)`. On `true` → `repository.save(order_id, "REFUNDED", amount_cents)`, then `notifier.notify(order_id, "Order {order_id} REFUNDED")`, return `true`. On `false` → nothing is saved or sent, return `false`.
8. `OrderProcessor::status(order_id) -> Option<String>` returns `repository.status_of(order_id)` (`None` for an unknown order).

## Examples

`meal = [Item { burger, qty 2, 800 }, Item { fries, qty 1, 300 }]` → items subtotal `1900`.

| Call | Result |
|---|---|
| `total(&meal, 2500, &DistanceBandFee::new())` | `1900 + 300 = 2200` |
| `total(&meal, 500, &DistanceBandFee::new())` | `1900 + 0 = 1900` |
| `total(&[], 500, &fee)` / an item with quantity 0 | `Err(OrderError::InvalidItems)` |
| `fee(1000)`, `fee(1001)`, `fee(3001)`, `fee(4001)` | `0`, `300`, `400`, `500` |

Placing orders (events in the order they happen):

| Call | Events | Returned |
|---|---|---|
| `place("o1", &meal, 2500, &fee, &approving("fake-1"))` | `charge:2200`, `save:o1:PAID:2200`, `notify:o1:Order o1 PAID` | `Ok(Placement { "PAID", 2200, "fake-1" })` |
| `place("o2", &meal, 500, &fee, &declining)` | `charge:1900`, `save:o2:PAYMENT_DECLINED:1900`, `notify:o2:Order o2 PAYMENT_DECLINED` | `Ok(Placement { "PAYMENT_DECLINED", 1900, "" })`; `status("o2")` = `Some("PAYMENT_DECLINED")` |
| `place("free", &[sample qty 1, 0 cents], 500, &fee, &p)` (delivery fee at 500 m is 0, total 0) | save + notify, no charge | `Ok(Placement { "PAID", 0, "" })` |
| `place(" ", …)` | nothing | `Err(OrderError::InvalidId)` |

Card walk-through with `CardPayment::new(10_000)`: `charge(1000)` → approved, `card-1`; `refund("card-1", 600)` → `true`; `refund("card-1", 500)` → `false` (would total 1100); `refund("card-1", 400)` → `true`; `refund("card-1", 1)` → `false`; `refund("card-99", 1)` → `false`. With `CardPayment::new(1000)`: `charge(10)` → `card-1`, `charge(5000)` → declined, next `charge(10)` → `card-2`.

Refund via the processor: after `place("o9", &meal, 2500, &fee, &card)` (reference `card-1`, paid 2200), `refund("o9","card-1",2000,&card)` → `true` (saves `REFUNDED`); `refund("o9","card-1",300,&card)` → `false` (2300 > 2200), nothing saved; `status("o9")` = `Some("REFUNDED")`.

## Edge cases

- A new fee policy or payment method written by a caller (e.g. a flat 999 fee, a voucher method) must work with `place` unchanged.
- Anything `Refundable` can be used for `refund`, even if it cannot charge.
- Declined payments are still saved and notified.

## Running

```bash
cargo test -p p51_solid_summary     # from rust/ — tests FAIL until you implement the stubs
cargo play run 51
```

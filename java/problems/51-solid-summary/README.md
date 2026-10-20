# 51 · SOLID Summary — Meal Order Checkout (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 75 min | Level 2 — value records and seven small interfaces are provided; the rest is yours to write. |

## Problem

**What you are building.** A food-delivery app takes an order (a list of meal items and how far away the customer is),
prices it (items plus a distance-based delivery fee), charges the customer through a payment method, records the outcome,
tells the customer, and can refund later. Payment methods, delivery-fee rules, the order store and the notifier are all
swappable: new ones must work with the checkout without the checkout changing. All money is in **cents** (`long`);
distances are in **metres** (`int`).

**Glossary**
- *Item* — `Item(name, quantity, unitCents)`: one order line, worth `quantity × unitCents`.
- *Fee policy* — a `DeliveryFeePolicy`: `fee(distanceMeters)` returns the delivery charge in cents.
- *Reference* — the id a payment method gives to an approved charge; it is later used to refund that charge.
- *Free order* — an order whose total is 0 cents.
- *Started kilometre* — any part of a kilometre counts as a whole one (3001 m is 1 m into the first kilometre past 3000, so it counts as one).

## Requirements

Provided in `contract/`: `Item`, `PaymentResult(approved, reference)`, `Placement(status, totalCents, reference)`, `PaymentMethod`, `Refundable`, `DeliveryFeePolicy`, `OrderRepository`, `Notifier`. You write `OrderPricer`, `DistanceBandFee`, `CardPayment`, `CashOnDelivery`, `OrderProcessor` in `solution/`.

1. `OrderPricer.total(items, distanceMeters, policy)` returns `Σ quantity × unitCents + policy.fee(distanceMeters)`. Null or empty `items`, any quantity `<= 0`, any negative unit price, negative distance, or null policy → `IllegalArgumentException`. A unit price of 0 is allowed.
2. `DistanceBandFee.fee(distanceMeters)`: 0–1000 m → 0; 1001–3000 m → 300; beyond 3000 m: 300 + 100 per *started* kilometre past 3000 (3001→400, 4000→400, 4001→500, 5000→500, 5001→600). Negative distance → `IllegalArgumentException`.
3. `CardPayment(limitCents)`: a charge up to the limit (inclusive) is approved with reference `card-1`, `card-2`, … (a counter that advances **only on approvals**); a larger charge is declined (`approved=false`, reference `""`). `refund(reference, amount)` returns `true` only when the reference is one this card issued **and** the cumulative refunds for that reference (including this one) stay `<=` the amount originally charged; otherwise `false` and nothing is recorded. It also implements `Refundable`.
4. `CashOnDelivery(maxCashCents)`: approves up to `maxCashCents` inclusive (references `cod-1`, `cod-2`, …, advancing only on approvals), declines more. It is **not** `Refundable`.
5. Every `PaymentMethod` follows the contract in `contract/PaymentMethod.java`: `charge` with an amount `<= 0` → `IllegalArgumentException` (exactly that type); any amount `> 0` (even `Long.MAX_VALUE`) never throws — it approves or declines; approved ⇒ non-blank, unique reference; declined ⇒ reference `""`.
6. `OrderProcessor(OrderPricer, OrderRepository, Notifier)`. `place(orderId, items, distanceMeters, fee, payment)`:
   - blank (null/empty/whitespace) `orderId` → `IllegalArgumentException`, before anything else happens (no charge, no save, no notify);
   - price the order with the pricer (its `IllegalArgumentException`s propagate);
   - if total `> 0`, call `payment.charge(total)`; approved → status `"PAID"` with the payment's reference, declined → status `"PAYMENT_DECLINED"` with reference `""`;
   - a free order (total 0) is `"PAID"` with reference `""` and the payment method is **never called**;
   - then, in this order: `repository.save(orderId, status, total)`, then `notifier.notify(orderId, "Order " + orderId + " " + status)`;
   - returns `Placement(status, totalCents, reference)`.
7. `OrderProcessor.refund(orderId, reference, amountCents, refundable)`: null `refundable` → `IllegalArgumentException`. Otherwise call `refundable.refund(reference, amountCents)`. On `true` → `repository.save(orderId, "REFUNDED", amountCents)`, then `notifier.notify(orderId, "Order " + orderId + " REFUNDED")`, return `true`. On `false` → nothing is saved or sent, return `false`.
8. `OrderProcessor.status(orderId)` returns `repository.statusOf(orderId)` (`Optional.empty()` for an unknown order).

## Examples

`MEAL = [Item("burger", 2, 800), Item("fries", 1, 300)]` → items subtotal `1900`.

| Call | Result |
|---|---|
| `total(MEAL, 2500, new DistanceBandFee())` | `1900 + 300 = 2200` |
| `total(MEAL, 500, new DistanceBandFee())` | `1900 + 0 = 1900` |
| `total(List.of(), 500, fee)` / quantity 0 / distance −1 | `IllegalArgumentException` |
| `fee(1000)`, `fee(1001)`, `fee(3001)`, `fee(4001)` | `0`, `300`, `400`, `500` |

Placing orders (events in the order they happen):

| Call | Events | Returned |
|---|---|---|
| `place("o1", MEAL, 2500, fee, approvingPayment("fake-1"))` | `charge:2200`, `save:o1:PAID:2200`, `notify:o1:Order o1 PAID` | `Placement("PAID", 2200, "fake-1")` |
| `place("o2", MEAL, 500, fee, decliningPayment)` | `charge:1900`, `save:o2:PAYMENT_DECLINED:1900`, `notify:o2:Order o2 PAYMENT_DECLINED` | `Placement("PAYMENT_DECLINED", 1900, "")`; `status("o2")` = `PAYMENT_DECLINED` |
| `place("free", [Item("sample",1,0)], 500, fee, p)` (delivery fee at 500 m is 0, total 0) | save + notify, no charge | `Placement("PAID", 0, "")` |
| `place(" ", …)` | nothing | `IllegalArgumentException` |

Card walk-through with `new CardPayment(10_000)`: `charge(1000)` → `card-1`; `refund("card-1", 600)` → `true`; `refund("card-1", 500)` → `false` (would total 1100); `refund("card-1", 400)` → `true`; `refund("card-1", 1)` → `false`; `refund("card-99", 1)` → `false`. With `new CardPayment(1000)`: `charge(10)` → `card-1`, `charge(5000)` → declined, next `charge(10)` → `card-2`.

Refund via the processor: after `place("o9", MEAL, 2500, fee, card)` (reference `card-1`, paid 2200), `refund("o9","card-1",2000,card)` → `true` (saves `REFUNDED`); `refund("o9","card-1",300,card)` → `false` (2300 > 2200), nothing saved; `status("o9")` = `REFUNDED`.

## Edge cases

- A new fee policy or payment method written by a caller (e.g. a flat 999 fee, a voucher method) must work with `place` unchanged.
- Anything `Refundable` can be used for `refund`, even if it cannot charge.
- Declined payments are still saved and notified.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

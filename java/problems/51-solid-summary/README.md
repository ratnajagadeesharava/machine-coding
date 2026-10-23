# 51 · SOLID Summary — Meal Order Checkout (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 75 min | Level 2 — value records and seven small interfaces are provided; the rest is yours to write. |

## Problem

A food-delivery app takes an order, prices it (items plus delivery), charges the customer, records the result, tells
the customer, and can refund later.

## Requirements

1. `OrderPricer.total(items, distanceMeters, policy)`: `Σ quantity × unitCents + policy.fee(distance)`. Null/empty items, quantity `<= 0`, negative price, negative distance or null policy → `IllegalArgumentException`.
2. `DistanceBandFee`: ≤ 1000 m → 0; ≤ 3000 m → 300; beyond: 300 + 100 per *started* kilometre past 3000 (3001→400, 4000→400, 4001→500). Negative → `IllegalArgumentException`.
3. `CardPayment(limit)`: approves charges up to the limit (inclusive) with references `card-1`, `card-2`, … (approvals only); larger ones are declined. Refund accepted when the reference is one it issued and the cumulative refunds for it stay within the amount charged.
4. `CashOnDelivery(maxCash)`: approves up to `maxCash` (references `cod-1`, …), declines more. **Not** `Refundable`.
5. Every `PaymentMethod` follows the contract in `contract/PaymentMethod.java` (zero/negative → `IllegalArgumentException`; never throws for a valid amount; approved ⇒ unique non-blank reference; declined ⇒ `""`).
6. `OrderProcessor.place(...)`: blank id → `IllegalArgumentException`. Price → (if total > 0) charge → **save → notify**, status `PAID` / `PAYMENT_DECLINED`. A free order (total 0) is `PAID` without calling the payment method (reference `""`). Returns `Placement(status, total, reference)`.
7. `OrderProcessor.refund(orderId, reference, amount, refundable)`: on `true` → save `REFUNDED`, then notify, return `true`; on `false` → nothing saved or sent. `status(orderId)` reads from the repository.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

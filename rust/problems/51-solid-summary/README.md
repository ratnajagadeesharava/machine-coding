# 51 · SOLID Summary — Meal Order Checkout (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 75 min | Level 2 — value types, six small traits and `OrderError` are provided in `src/contract.rs`; the rest is yours to write in `src/lib.rs`. |

## Problem

A food-delivery app takes an order, prices it (items plus delivery), charges the customer, records the result, tells
the customer, and can refund later.

## Requirements

1. `OrderPricer::total(items, distance_m, policy)`: `Σ quantity × unit_cents + policy.fee(distance)`. Empty items or a quantity of 0 → `Err(OrderError::InvalidItems)`.
2. `DistanceBandFee`: ≤ 1000 m → 0; ≤ 3000 m → 300; beyond: 300 + 100 per *started* kilometre past 3000 (3001→400, 4000→400, 4001→500).
3. `CardPayment::new(limit)`: approves charges up to the limit (inclusive) with references `card-1`, `card-2`, … (approvals only); larger ones are declined. Refund accepted when the reference is one it issued and the cumulative refunds for it stay within the amount charged.
4. `CashOnDelivery::new(max_cash)`: approves up to `max_cash` (references `cod-1`, …), declines more. Does **not** implement `Refundable`.
5. Every `PaymentMethod` follows the contract in `src/contract.rs` (`charge(0)` → `Err(InvalidAmount)`; never fails for an amount > 0; approved ⇒ unique non-blank reference; declined ⇒ `""`).
6. `OrderProcessor::place(...)`: blank id → `Err(InvalidId)`. Price → (if total > 0) charge → **save → notify**, status `PAID` / `PAYMENT_DECLINED`. A free order (total 0) is `PAID` without calling the payment method (reference `""`).
7. `OrderProcessor::refund(order_id, reference, amount, &dyn Refundable)`: on `true` → save `REFUNDED`, then notify, return `true`; on `false` → nothing saved or sent. `status(order_id)` reads from the repository.

## Running

```bash
cargo test -p p51_solid_summary     # from rust/ — tests FAIL until you implement the stubs
cargo play run 51
```

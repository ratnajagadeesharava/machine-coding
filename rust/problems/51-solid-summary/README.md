# 51 · SOLID Summary — Meal Order Checkout (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 75 min | Level 2 — value types, six small traits and `OrderError` are provided in `src/contract.rs`; five types are stubs in `src/lib.rs`. |

> **One mini-system, all five principles.** Each type below exists to demonstrate one principle; the tests are
> built so that a design that breaks any of them fails.

| Principle | Where it shows up |
|---|---|
| **S**ingle Responsibility | `OrderPricer` only prices; `OrderProcessor` only coordinates; storing and notifying are other types' jobs. |
| **O**pen/Closed | New `DeliveryFeePolicy` / `PaymentMethod` implementors (`FlatFee`, `Voucher`, written in the tests) work with no edits. |
| **L**iskov Substitution | `CardPayment` and `CashOnDelivery` both pass one shared `payment_contract` suite. |
| **I**nterface Segregation | `Refundable` is a separate trait: cash does not implement it, and `refund` asks for `&dyn Refundable`, not a whole payment method. |
| **D**ependency Inversion | `OrderProcessor::new` receives `OrderPricer`, `Box<dyn OrderRepository>`, `Box<dyn Notifier>`; fakes are injected. |

## Problem

A food-delivery app takes an order, prices it (items plus delivery), charges the customer, records the result, tells
the customer, and can refund later.

## Requirements

1. `OrderPricer::total(items, distance_m, policy)`: `Σ quantity × unit_cents + policy.fee(distance)`. Empty items or a quantity of 0 → `Err(OrderError::InvalidItems)`.
2. `DistanceBandFee`: ≤ 1000 m → 0; ≤ 3000 m → 300; beyond: 300 + 100 per *started* kilometre past 3000 (3001→400, 4000→400, 4001→500).
3. `CardPayment::new(limit)`: approves charges up to the limit (inclusive) with references `card-1`, `card-2`, … (approvals only); larger ones are declined. Refund accepted when the reference is one it issued and the cumulative refunds for it stay within the amount charged.
4. `CashOnDelivery::new(max_cash)`: approves up to `max_cash` (references `cod-1`, …), declines more. Does **not** implement `Refundable`.
5. Every `PaymentMethod` follows the contract in `src/contract.rs` (`charge(0)` → `Err(InvalidAmount)`; never fails for an amount > 0; approved ⇒ unique non-blank reference; declined ⇒ `""`). `charge` takes `&self`, so use `Cell`/`RefCell` for counters.
6. `OrderProcessor::place(...)`: blank id → `Err(InvalidId)`. Price → (if total > 0) charge → **save → notify**, status `PAID` / `PAYMENT_DECLINED`. A free order (total 0) is `PAID` without calling the payment method (reference `""`).
7. `OrderProcessor::refund(order_id, reference, amount, &dyn Refundable)`: on `true` → save `REFUNDED`, then notify, return `true`; on `false` → nothing saved or sent. `status(order_id)` reads from the repository.

## Running

```bash
cargo test -p p51_solid_summary     # from rust/ — tests FAIL until you implement the stubs
cargo play run 51
```

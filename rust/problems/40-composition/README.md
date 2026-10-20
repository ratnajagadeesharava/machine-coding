# 40 · Composition (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — `LineItem` (with a live-instance counter) and `OrderError` are provided; you write `Order`. |

## Problem

An order is made of line items. A line item has no meaning without its order: it must not be created on its own,
shared between orders, or outlive the order.

## Requirements

1. `Order::new(id)`; `add_line(sku, quantity, unit_price)` adds a line to the order.
2. Blank sku, zero quantity or zero price → `Err(OrderError::InvalidLine)`. An SKU that is already on the order adds to that line's quantity; a different unit price → `Err(OrderError::PriceMismatch)` and nothing changes.
3. `lines()`, `total()` = sum of `subtotal()`s.
4. `remove_line(sku)` removes one line, returning whether it existed.
5. `cancel()` removes all lines; afterwards `total()` is 0, `is_cancelled()` is true and `add_line` returns `Err(OrderError::Cancelled)`.
6. **Lifecycle:** the tests read `LineItem::live_count()` to prove that lines are destroyed when the order is dropped, when a line is removed and when the order is cancelled.

## Running

```bash
cargo test -p p40_composition     # from rust/ — tests FAIL until you implement Order
cargo play run 40
```

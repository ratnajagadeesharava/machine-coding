# 40 · Composition (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — `Order` is a stub in `src/lib.rs`. `LineItem` (with a live-instance counter) and `OrderError` are provided in `src/contract.rs`. |

> **Concept:** composition is a *has-a* relationship where the whole **owns** its parts. In Rust that is plain
> ownership: `Order` holds `Vec<LineItem>` by value, so dropping the order drops every line. Nothing else can
> create or share a line.

## Problem

An order is made of line items. A line item has no meaning without its order: it must not be created on its own,
shared between orders, or outlive the order.

## Requirements

1. `Order::new(id)`; `add_line(sku, quantity, unit_price)` creates the `LineItem` **inside** the order using `LineItem::new` (only code in this crate can call it).
2. Blank sku, zero quantity or zero price → `Err(OrderError::InvalidLine)`. An SKU that is already on the order adds to that line's quantity (`LineItem::add_quantity`); a different unit price → `Err(OrderError::PriceMismatch)` and nothing changes.
3. `lines()`, `total()` = sum of `subtotal()`s.
4. `remove_line(sku)` removes and destroys one line, returning whether it existed.
5. `cancel()` destroys all lines; afterwards `total()` is 0, `is_cancelled()` is true and `add_line` returns `Err(OrderError::Cancelled)`.
6. **Lifecycle:** the tests read `LineItem::live_count()` to prove that lines are destroyed when the order is dropped, when a line is removed and when the order is cancelled.

## Constraints

- **Rust adaptation:** Java uses weak references + GC to observe destruction; Rust is deterministic, so a drop counter does the same job. The counter lives in the provided `LineItem`; you only decide who owns the lines.
- Do not wrap lines in `Rc`/`Arc` or keep them in a `static`.

## Running

```bash
cargo test -p p40_composition     # from rust/ — tests FAIL until you implement Order
cargo play run 40
```

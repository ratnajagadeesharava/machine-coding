# 40 · Composition (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — `Order` and `LineItem` stubs with the signatures; you write the bodies. |

## Problem

An order is made of line items. A line item has no meaning without its order: it must not be created on its own,
shared between orders, or left alive after the order is gone.

## Requirements

1. `new Order(id)`; `addLine(sku, quantity, unitPriceCents)` adds a line to the order.
2. Invalid input (blank sku, quantity or price `<= 0`) → `IllegalArgumentException`. Adding an SKU that already exists adds to that line's quantity; a different unit price for the same SKU → `IllegalArgumentException` and nothing changes.
3. `lineCount()`, `lineAt(i)` (same object every call), `total()` = sum of `subtotal()`s.
4. `removeLine(sku)` removes one line, returning whether it existed.
5. `cancel()` removes all lines; afterwards `lineCount()` and `total()` are 0, `isCancelled()` is true and `addLine` throws `IllegalStateException`.
6. The tests check by reflection that `LineItem` has no public constructor and that no public `Order` method takes a `LineItem`.
7. After the order is unreachable, or a line is removed/cancelled, the `LineItem` object must be garbage-collectable. The tests use `WeakReference` + `System.gc()`.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

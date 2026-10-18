# 40 · Composition (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — `Order` and `LineItem` stubs. Bodies throw `UnsupportedOperationException`. |

> **Concept:** composition is a *has-a* relationship where the whole **owns** its parts. The owner creates them,
> nobody else can create or share them, and **they die with the owner**.

## Problem

An order is made of line items. A line item has no meaning without its order: it must not be created on its own,
shared between orders, or left alive after the order is gone.

## Requirements

1. `new Order(id)`; `addLine(sku, quantity, unitPriceCents)` creates the `LineItem` **inside** the order.
2. Invalid input (blank sku, quantity or price `<= 0`) → `IllegalArgumentException`. Adding an SKU that already exists adds to that line's quantity; a different unit price for the same SKU → `IllegalArgumentException` and nothing changes.
3. `lineCount()`, `lineAt(i)` (the order's own instance, same object every call), `total()` = sum of `subtotal()`s.
4. `removeLine(sku)` removes and destroys one line, returning whether it existed.
5. `cancel()` destroys all lines; afterwards `lineCount()` and `total()` are 0, `isCancelled()` is true and `addLine` throws `IllegalStateException`.
6. **Ownership is enforced by the design:** `LineItem` has **no public constructor**, and no public `Order` method takes a `LineItem`. The tests check both by reflection.
7. **Lifecycle:** after the order is unreachable, or a line is removed/cancelled, the `LineItem` object must be garbage-collectable. The tests use `WeakReference` + `System.gc()`. Do not keep lines in any static registry or cache.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

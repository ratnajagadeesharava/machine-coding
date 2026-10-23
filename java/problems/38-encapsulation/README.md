# 38 · Encapsulation (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 30 min | Level 2 — a `StockItem` stub with the signatures; you write the bodies. |

## Problem

A warehouse tracks the stock of each product. Bugs have been traced to code that changed quantities directly,
pushed stock above shelf capacity and edited a shared tag list. Build `StockItem` so that **no sequence of public
calls can put it in an invalid state**.

## Requirements

1. `new StockItem(sku, capacity)`: blank sku or `capacity <= 0` → `IllegalArgumentException`. Starts with quantity 0.
2. `add(n)` / `remove(n)`: `n <= 0` → `IllegalArgumentException`. Going above `capacity` or below 0 → `IllegalStateException` and **nothing changes**.
3. `transferTo(other, n)` is atomic: if either item would break its rules, neither changes.
4. Tags: `addTag` trims and lower-cases; blank and duplicate tags are ignored. Changing the list returned by `tags()` never changes the item.
5. `history()` records every *successful* change in order (`"+5"`, `"-2"`, `"-1 (transfer)"` for the source, `"+1 (transfer)"` for the target) and is read-only.
6. The tests check with reflection that all fields are `private` and that there are no `setXxx` methods.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement StockItem
```

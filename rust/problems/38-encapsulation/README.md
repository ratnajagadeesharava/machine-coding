# 38 · Encapsulation (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 30 min | Level 2 — a `StockItem` stub and `StockError` are provided; you write the bodies. |

## Problem

A warehouse tracks the stock of each product. Bugs have been traced to code that changed quantities directly and
pushed stock above shelf capacity. Build `StockItem` so that **no sequence of public calls can put it in an
invalid state**.

## Requirements

1. `StockItem::new(sku, capacity)`: blank sku or `capacity == 0` → `Err(StockError::InvalidInput)`. Starts empty.
2. `add(n)` / `remove(n)`: `n == 0` → `Err(InvalidInput)`. Going above `capacity` or below 0 → `Err(InvariantViolation)` and **nothing changes**.
3. `transfer_to(&mut other, n)` is atomic: if either item would break its rules, neither changes (and neither history changes).
4. `add_tag` trims and lower-cases; blank and duplicate tags are ignored. `tags()` returns a read-only slice.
5. `history()` records every *successful* change in order (`"+5"`, `"-2"`, `"-1 (transfer)"` for the source, `"+1 (transfer)"` for the target).

## Running

```bash
cargo test -p p38_encapsulation     # from rust/ — tests FAIL until you implement StockItem
cargo play run 38
```

# 38 · Encapsulation (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 30 min | Level 2 — one stub type, `StockItem`, with `todo!()` bodies; `StockError` is provided. |

> **Concept:** keep state private and expose behaviour, so the object's rules (invariants) cannot be broken from
> outside. In Rust, fields are private by default; do not make them `pub`.

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

## Constraints

- No `pub` fields. The tests only use the methods above, so an invalid state must be unreachable through them.
- **Rust adaptation:** you cannot reflect over fields, so privacy is checked by the compiler instead; returning `&[String]` is how Rust hands out a view that callers cannot change.

## Running

```bash
cargo test -p p38_encapsulation     # from rust/ — tests FAIL until you implement StockItem
cargo play run 38
```

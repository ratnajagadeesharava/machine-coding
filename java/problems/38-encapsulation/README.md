# 38 · Encapsulation (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 30 min | Level 2 — one stub class, `StockItem`. Bodies throw `UnsupportedOperationException`. |

> **Concept:** keep state private and expose behaviour, so the object's rules (invariants) cannot be broken from
> outside. The only way to change the object is through methods that check the rules.

## Problem

A warehouse tracks the stock of each product. Bugs have been traced to code that changed quantities directly,
pushed stock above shelf capacity and edited a shared tag list. Build `StockItem` so that **no sequence of public
calls can put it in an invalid state**.

## Requirements

1. `new StockItem(sku, capacity)`: blank sku or `capacity <= 0` → `IllegalArgumentException`. Starts with quantity 0.
2. `add(n)` / `remove(n)`: `n <= 0` → `IllegalArgumentException`. Going above `capacity` or below 0 → `IllegalStateException` and **nothing changes**.
3. `transferTo(other, n)` is atomic: if either item would break its rules, neither changes.
4. Tags: `addTag` trims and lower-cases; blank and duplicate tags are ignored. `tags()` must not leak the internal list: changing what it returns never changes the item.
5. `history()` records every *successful* change in order (`"+5"`, `"-2"`, `"-1 (transfer)"` for the source, `"+1 (transfer)"` for the target) and is read-only.
6. All fields are `private` and there are no `setXxx` methods. The tests check this with reflection.

## Constraints

- The class is the only place the rules are enforced. Callers cannot reach the state except through methods.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement StockItem
```

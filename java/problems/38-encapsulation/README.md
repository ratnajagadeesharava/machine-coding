# 38 · Encapsulation (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 30 min | Level 2 — a `StockItem` stub with the signatures; you write the bodies. |

## Problem

You are building the stock record for one product in a warehouse. Bugs have been traced to code that changed
quantities directly, pushed stock above shelf capacity and edited a shared tag list. Build `StockItem` so that
**no sequence of public calls can put it in an invalid state**: the quantity is always between 0 and the shelf
capacity, tags stay tidy, and the audit history can't be rewritten. Items can also hand stock to each other
(a transfer between shelves).

## Glossary

- **sku**: the product's identifier string.
- **capacity**: the maximum quantity the shelf can hold.
- **quantity**: units currently in stock, always `0 <= quantity <= capacity`.
- **transfer**: moving `n` units from one item (source) to another (target).

## Requirements

1. `new StockItem(sku, capacity)`: blank (null, empty or whitespace-only) sku or `capacity <= 0` throws `IllegalArgumentException`. Starts with quantity 0. Accessors: `sku()`, `capacity()`, `quantity()`.
2. `add(n)` / `remove(n)`: `n <= 0` throws `IllegalArgumentException`. Going above `capacity` or below 0 throws `IllegalStateException` and **nothing changes**. Reaching exactly `capacity` or exactly 0 is allowed.
3. `transferTo(other, n)` removes `n` from this item and adds `n` to `other`. It is atomic: if either item would break its rules (source too low, target too full), it throws `IllegalStateException` and neither quantity nor either history changes.
4. Tags: `addTag` trims and lower-cases; blank and duplicate tags (after normalising) are ignored. `tags()` lists them in the order first added. Changing the list returned by `tags()` never changes the item (it may be a read-only or a mutable copy).
5. `history()` records every *successful* change in order (`"+5"`, `"-2"`, `"-1 (transfer)"` for the source, `"+1 (transfer)"` for the target) and is read-only: modifying it throws `UnsupportedOperationException`. Failed operations leave no entry.
6. The tests check with reflection that all fields are `private` and that there are no `setXxx` methods.

## Examples

```java
StockItem a = new StockItem("A", 10);
StockItem b = new StockItem("B", 10);
a.add(5);              // quantity 5
a.remove(2);           // quantity 3
a.remove(50);          // IllegalStateException, quantity still 3, no history entry
a.transferTo(b, 1);    // a = 2, b = 1
a.history();           // ["+5", "-2", "-1 (transfer)"]
b.history();           // ["+1 (transfer)"]

StockItem from = new StockItem("A", 10), to = new StockItem("B", 5);
from.add(8); to.add(3);
from.transferTo(to, 4); // target would reach 7 > 5: IllegalStateException, from=8, to=3
from.transferTo(to, 2); // ok: from=6, to=5

StockItem t = new StockItem("S", 10);
t.addTag("Fragile"); t.addTag("  fragile "); t.addTag(""); t.addTag("Heavy");
t.tags();              // ["fragile", "heavy"]
```

## Edge cases

- `add(0)`, `add(-1)`, `remove(0)` throw `IllegalArgumentException`; `new StockItem(" ", 5)` and `new StockItem("S", 0)` too.
- `remove(9)` on quantity 8 throws `IllegalStateException`; quantity stays 8.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement StockItem
```

# 38 · Encapsulation (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 30 min | Level 2 — a `StockItem` stub and `StockError` are provided; you write the bodies. |

## Problem

You are building the stock record for one product in a warehouse. Bugs have been traced to code that changed
quantities directly and pushed stock above shelf capacity. Build `StockItem` so that **no sequence of public calls
can put it in an invalid state**: the quantity is always between 0 and the shelf capacity, tags stay tidy, and the
audit history can't be rewritten. Items can also hand stock to each other (a transfer between shelves).

## Glossary

- **sku**: the product's identifier string.
- **capacity**: the maximum quantity the shelf can hold (`u32`).
- **quantity**: units currently in stock, always `0 <= quantity <= capacity`.
- **transfer**: moving `n` units from one item (source) to another (target).

## Requirements

1. `StockItem::new(sku, capacity)`: blank (empty or whitespace-only) sku or `capacity == 0` gives `Err(StockError::InvalidInput)`. Starts empty (quantity 0). Accessors: `sku()`, `capacity()`, `quantity()`.
2. `add(n)` / `remove(n)`: `n == 0` gives `Err(InvalidInput)`. Going above `capacity` or below 0 gives `Err(InvariantViolation)` and **nothing changes**. Reaching exactly `capacity` or exactly 0 is allowed.
3. `transfer_to(&mut other, n)` removes `n` from this item and adds `n` to `other`. It is atomic: if either item would break its rules (source too low, target too full), it returns `Err(InvariantViolation)` and neither quantity nor either history changes.
4. `add_tag` trims and lower-cases; blank and duplicate tags (after normalising) are ignored. `tags()` returns a read-only slice in the order first added.
5. `history()` records every *successful* change in order (`"+5"`, `"-2"`, `"-1 (transfer)"` for the source, `"+1 (transfer)"` for the target) and is read-only (a slice of `String`). Failed operations leave no entry.

## Examples

```rust
let mut a = StockItem::new("A", 10).unwrap();
let mut b = StockItem::new("B", 10).unwrap();
a.add(5).unwrap();            // quantity 5
a.remove(2).unwrap();         // quantity 3
a.remove(50);                 // Err(InvariantViolation), quantity still 3, no history entry
a.transfer_to(&mut b, 1).unwrap(); // a = 2, b = 1
a.history();                  // ["+5", "-2", "-1 (transfer)"]
b.history();                  // ["+1 (transfer)"]

let mut from = StockItem::new("A", 10).unwrap();
let mut to = StockItem::new("B", 5).unwrap();
from.add(8).unwrap(); to.add(3).unwrap();
from.transfer_to(&mut to, 4); // target would reach 7 > 5: Err(InvariantViolation), from=8, to=3
from.transfer_to(&mut to, 2).unwrap(); // from=6, to=5

let mut t = StockItem::new("S", 10).unwrap();
t.add_tag("Fragile"); t.add_tag("  fragile "); t.add_tag(""); t.add_tag("Heavy");
t.tags();                     // ["fragile", "heavy"]
```

## Edge cases

- `add(0)` and `remove(0)` give `Err(InvalidInput)`; `StockItem::new(" ", 5)` and `StockItem::new("S", 0)` too.
- `remove(9)` on quantity 8 gives `Err(InvariantViolation)`; quantity stays 8.

## Running

```bash
cargo test -p p38_encapsulation     # from rust/ — tests FAIL until you implement StockItem
cargo play run 38
```

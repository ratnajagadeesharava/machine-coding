# 40 · Composition (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — `LineItem` (with a live-instance counter) and `OrderError` are provided; you write `Order`. |

## Problem

You are building the order model of a shop. An order is made of line items (SKU, quantity, unit price). A line item
has no meaning without its order: nobody outside the order can create one, share one between orders, or keep one
alive after the order that owns it is gone. Order totals are in cents.

## Glossary

- **line item**: one row of an order: a SKU, a quantity (`u32`) and a unit price in cents (`u64`).
- **subtotal**: `quantity x unit_price` of one line.
- **SKU**: the product code that identifies a line within an order; an order has at most one line per SKU.
- **cancelled**: a terminal state of the order; it holds no lines and accepts no new ones.
- **live count**: `LineItem::live_count()`, the number of `LineItem`s currently alive in the whole process (provided).

## Requirements

1. `Order::new(id)` with `id()`; `add_line(sku, quantity, unit_price)` adds a line to the order. Lines keep the order in which their SKUs were first added.
2. Blank sku, zero quantity or zero price gives `Err(OrderError::InvalidLine)` and nothing is added. An SKU that is already on the order adds to that line's quantity (one line, not two); a different unit price gives `Err(OrderError::PriceMismatch)` and nothing changes.
3. `lines()` lists the lines (each with `sku()`, `quantity()`, `subtotal()`); `total()` = sum of `subtotal()`s (0 when empty).
4. `remove_line(sku)` removes that SKU's line and returns `true`; it returns `false` if there was no such line.
5. `cancel()` removes all lines; afterwards `lines()` is empty, `total()` is 0, `is_cancelled()` is true and `add_line` returns `Err(OrderError::Cancelled)`.
6. **Lifecycle:** the tests read `LineItem::live_count()` to prove that lines are destroyed when the order is dropped, when a line is removed and when the order is cancelled, and that two orders never share a line.

## Examples

```rust
let mut o = Order::new("o-1");
o.add_line("A", 2, 500).unwrap();
o.add_line("B", 1, 250).unwrap();
o.lines().len();           // 2
o.total();                 // 1250 (2 x 500 + 250)
o.lines()[0].subtotal();   // 1000

o.add_line("A", 3, 500).unwrap();  // merges: still 2 lines, line A quantity 5
o.add_line("A", 1, 999);           // Err(PriceMismatch), quantity stays 5

o.remove_line("A");        // true
o.remove_line("A");        // false

// lifecycle (before = LineItem::live_count() at the start)
// after adding 3 lines: before + 3;  after drop(order): before
// after cancel(): live_count back to before; add_line("C", 1, 1) -> Err(Cancelled)
```

## Edge cases

- `add_line("", 1, 100)`, `add_line("A", 0, 100)` and `add_line("A", 1, 0)` all give `Err(InvalidLine)`; no `LineItem` is created (`live_count()` unchanged).
- Dropping one order does not affect the lines of another order with the same SKU.

## Running

```bash
cargo test -p p40_composition     # from rust/ — tests FAIL until you implement Order
cargo play run 40
```

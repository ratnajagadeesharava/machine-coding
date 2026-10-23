# 40 · Composition (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — `Order` and `LineItem` stubs with the signatures; you write the bodies. |

## Problem

You are building the order model of a shop. An order is made of line items (SKU, quantity, unit price). A line item
has no meaning without its order: nobody outside the order can create one, hand one in, share one between orders, or
keep one alive after the order that owns it is gone. Order totals are in cents.

## Glossary

- **line item**: one row of an order: a SKU, a quantity and a unit price in cents.
- **subtotal**: `quantity x unitPrice` of one line.
- **SKU**: the product code that identifies a line within an order; an order has at most one line per SKU.
- **cancelled**: a terminal state of the order; it holds no lines and accepts no new ones.

## Requirements

1. `new Order(id)` with `id()`; `addLine(sku, quantity, unitPriceCents)` adds a line to the order. Lines keep the order in which their SKUs were first added.
2. Invalid input (blank sku, quantity `<= 0` or price `<= 0`) throws `IllegalArgumentException` and nothing is added. Adding an SKU that already exists adds to that line's quantity (one line, not two); a different unit price for the same SKU throws `IllegalArgumentException` and nothing changes.
3. `lineCount()`, `lineAt(i)` (the same object every call for the same index) with `sku()`, `quantity()` and `subtotal()`, and `total()` = sum of the lines' `subtotal()`s (0 when empty).
4. `removeLine(sku)` removes that SKU's line and returns `true`; it returns `false` if there was no such line. Remaining lines are unaffected.
5. `cancel()` removes all lines; afterwards `lineCount()` and `total()` are 0, `isCancelled()` is true and `addLine` throws `IllegalStateException`.
6. The tests check by reflection that `LineItem` has no public constructor and that no public `Order` method takes a `LineItem`.
7. After the order is unreachable, or a line is removed/cancelled, the `LineItem` object must be garbage-collectable. The tests use `WeakReference` + `System.gc()`.

## Examples

```java
Order o = new Order("o-1");
o.addLine("A", 2, 500);
o.addLine("B", 1, 250);
o.lineCount();             // 2
o.total();                 // 1250 (2 x 500 + 250)
o.lineAt(0).subtotal();    // 1000

o.addLine("A", 3, 500);    // merges: still 2 lines, line A quantity 5
o.addLine("A", 1, 999);    // IllegalArgumentException (price differs), quantity stays 5

o.removeLine("A");         // true
o.removeLine("A");         // false
o.total();                 // 250

o.cancel();
o.isCancelled();           // true
o.total();                 // 0
o.addLine("C", 1, 1);      // IllegalStateException
```

## Edge cases

- `addLine("", 1, 100)`, `addLine("A", 0, 100)` and `addLine("A", 1, 0)` throw `IllegalArgumentException`; `lineCount()` stays 0.
- After `removeLine("A")`, the line that was at index 1 is now at index 0.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

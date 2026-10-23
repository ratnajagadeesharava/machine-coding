# 46 · SRP — Single Responsibility Principle (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — three small interfaces and two value records are provided; you write two implementations and a service. |

## Problem

Invoice issuing used to be one 400-line `InvoiceManager` that added up lines, built the text and wrote to the
database. Every change broke something else.

## Requirements

1. `StandardCalculator` (implements `TotalsCalculator`): `subtotal = Σ quantity × unitCents`; `tax = subtotal × taxBps / 10000` rounded **half up**; `total = subtotal + tax`. Empty lines, quantity `<= 0`, negative unit price or negative `taxBps` → `IllegalArgumentException`.
2. `TextRenderer` (implements `InvoiceRenderer`): the exact layout in its Javadoc. It prints the totals **it is given**, even if they do not match the lines.
3. `InvoiceService(calculator, renderer, repository)`:
   - `issue(id, lines, taxBps)`: blank id → `IllegalArgumentException` before anything is called; otherwise **calculate → render → save** in that order, returning the rendered text.
   - If the calculator throws, the exception propagates and the renderer and repository are never called.
   - `lookup(id)` reads from the repository.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

# 46 · SRP — Single Responsibility Principle (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — value types, three single-purpose traits and `InvoiceError` are provided in `src/contract.rs`; you write two implementations and a service. |

## Problem

Invoice issuing used to be one 400-line `InvoiceManager` that added up lines, built the text and wrote to the
database. Every change broke something else.

## Requirements

1. `StandardCalculator` (implements `TotalsCalculator`): `subtotal = Σ quantity × unit_cents`; `tax = subtotal × tax_bps / 10000` rounded **half up**; `total = subtotal + tax`. Empty lines or a quantity of 0 → `Err(InvoiceError::InvalidLines)`.
2. `TextRenderer` (implements `InvoiceRenderer`): the exact layout in its doc comment. It prints the totals **it is given**, even if they do not match the lines.
3. `InvoiceService::new(calculator, renderer, repository)`:
   - `issue(id, lines, tax_bps)`: blank id → `Err(InvoiceError::InvalidId)` before anything is called; otherwise **calculate → render → save** in that order, returning the rendered text.
   - If the calculator fails, its error is returned and the renderer and repository are never called.
   - `lookup(id)` reads from the repository.

## Running

```bash
cargo test -p p46_srp     # from rust/ — tests FAIL until you implement the stubs
cargo play run 46
```

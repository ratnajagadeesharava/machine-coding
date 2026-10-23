# 46 · SRP — Single Responsibility Principle (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — value types, three single-purpose traits and `InvoiceError` are provided in `src/contract.rs`; you write two implementations and a service. |

## Problem

**What you are building.** A billing back end issues invoices. Issuing one means three separate jobs: working out the money (subtotal, tax, total), turning the invoice into printable text, and storing that text so it can be looked up later. This used to be one 400-line `InvoiceManager` that did all three, and every change broke something else. You rebuild it as three single-purpose parts (calculating, rendering, and an `InvoiceService` that issues invoices using them) and check that each does only its own job.

## Requirements

Provided in `src/contract.rs`: structs `InvoiceLine { description, quantity: u32, unit_cents: u64 }` and `InvoiceTotals { subtotal, tax, total }` (all amounts are whole cents), `InvoiceError { InvalidLines, InvalidId }`, and traits `TotalsCalculator::calculate(&self, lines, tax_bps: u64)`, `InvoiceRenderer::render(&self, invoice_id, lines, totals)` and `InvoiceRepository::save(&self, id, text)` / `find(&self, id)` (returns `Option<String>`). You write `StandardCalculator`, `TextRenderer` and `InvoiceService`.

1. `StandardCalculator::new()` (implements `TotalsCalculator`): `subtotal = Σ quantity × unit_cents`; `tax = subtotal × tax_bps / 10000` rounded **half up** to a whole cent; `total = subtotal + tax`. `tax_bps` is the tax rate in basis points (1800 = 18.00%). Empty `lines` or any `quantity` of 0 → `Err(InvoiceError::InvalidLines)`. A `tax_bps` of 0 is valid (tax 0).
2. `TextRenderer::new()` (implements `InvoiceRenderer`): produces the exact layout shown below (also in its doc comment). It prints the totals **it is given**, even if they do not match the lines; it does no arithmetic on the totals.
3. `InvoiceService::new(calculator, renderer, repository)`:
   - `issue(id, lines, tax_bps)`: blank id (empty or whitespace only) → `Err(InvoiceError::InvalidId)` before anything is called; otherwise **calculate → render → save** in that order, returning `Ok(rendered text)`. The renderer receives the exact totals the calculator returned; the repository saves under `id` the exact text the renderer returned.
   - If the calculator fails, its error is returned unchanged and the renderer and repository are never called (nothing is saved).
   - `lookup(id)` returns what the repository's `find(id)` gives (`None` when the repository has no such invoice).

### Rendered layout

Lines are joined with `"\n"`, with no trailing newline. One line per invoice line, in the order given, then three total lines:

```
INVOICE A-1
Widget x2 @ 5.00 = 10.00
Gadget x1 @ 2.50 = 2.50
SUBTOTAL 12.50
TAX 2.25
TOTAL 14.75
```

Item line format: `<description> x<quantity> @ <unit price> = <line total>`. Money is formatted with exactly two decimals: 1250 cents is `12.50`, 1 cent is `0.01`, 1000 cents is `10.00`.

## Examples

Calculator:

```
calculator.calculate(&[Widget x2 @ 500, Gadget x1 @ 250], 1800)
   -> Ok(InvoiceTotals { subtotal: 1250, tax: 225, total: 1475 })   // 1250 * 1800 / 10000 = 225 exactly
calculator.calculate(&[x x1 @ 1005], 1800).tax -> 181               // 180.9 rounds up
calculator.calculate(&[x x1 @ 5], 1000).tax    -> 1                 // 0.5 rounds half up
calculator.calculate(&[], 1800)                 -> Err(InvalidLines)
calculator.calculate(&[x x0 @ 10], 1800)        -> Err(InvalidLines)
```

Renderer (totals are printed as given, not recomputed):

```
render("A-2", [Widget x2 @ 500, Gadget x1 @ 250], totals = (subtotal 999, tax 1, total 1000))
   -> "INVOICE A-2\nWidget x2 @ 5.00 = 10.00\nGadget x1 @ 2.50 = 2.50\nSUBTOTAL 9.99\nTAX 0.01\nTOTAL 10.00"
```

Service:

```
service = InvoiceService::new(Box::new(calc), Box::new(renderer), Box::new(repo))
service.issue("A-1", &lines, 1800)
   -> calls, in order: calc.calculate(lines, 1800), renderer.render("A-1", lines, totals), repo.save("A-1", text)
   -> returns Ok(text)
service.lookup("A-1")  -> Some(text)       service.lookup("nope") -> None
service.issue(" ", &lines, 1800) -> Err(InvalidId), no collaborator called
calc returns Err(InvalidLines)   -> issue returns Err(InvalidLines); renderer and repo never called
```

## Edge cases

- Blank means empty or only whitespace.
- The id check happens first, so a blank id never reaches the calculator even if the lines are also invalid.
- A failed issue saves nothing: `lookup` of that id stays empty.

## Glossary

- **Cents**: all amounts are whole cents (500 = 5.00).
- **Basis points (bps)**: hundredths of a percent; `tax_bps / 10000` is the tax rate (1800 bps = 18%).
- **Half up**: a fraction of exactly one half rounds up to the next whole cent; below half rounds down.
- **Repository**: the place issued invoice text is stored and looked up by invoice id.

## Running

```bash
cargo test -p p46_srp     # from rust/ — tests FAIL until you implement the stubs
cargo play run 46
```

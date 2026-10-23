# 42 · DRY — Don't Repeat Yourself (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 30 min | Level 2 — the `PricingRules` trait and `DocError` are provided; you write `StandardRules` and `Documents`. |

## Problem

You are building the document printing of a shop. It prints three documents: a **quote** (an estimate), an
**invoice** (final) and a **credit note** (a refund). They all use the same pricing (bulk discount, then tax) and the
same money formatting. Last quarter the tax rate changed and the credit notes were still using the old one: after
your change, changing a pricing rule must change all three documents consistently.

## Glossary

- **cents**: all amounts are whole cents in `i64`; lines are the item prices of the document.
- **bps (basis points)**: hundredths of a percent; 1800 bps = 18%, 10000 bps = 100%.
- **net**: the sum of the lines.
- **bulk discount**: a percentage off the net, applied only when the net reaches the threshold.
- **taxable**: `net - discount`; tax is charged on this.
- **round half up**: a fractional cent of exactly `.5` or more rounds up (180.9 becomes 181; 0.9 becomes 1).
- **`PricingRules`** (provided): `discount(net)` and `tax(taxable)`, both in cents, both `>= 0`.

## Requirements

1. `StandardRules::new(tax_bps, bulk_threshold, bulk_discount_bps)`: `discount(net)` = `net * bulk_discount_bps / 10000` when `net >= bulk_threshold`, else 0; `tax(taxable)` = `taxable * tax_bps / 10000`. Both **round half up** to whole cents.
2. `Documents::new(Box<dyn PricingRules>)` produces the strings below, using whatever rules it was given (the tests supply their own rules too):

   ```
   quote(lines)                "QUOTE net=50.00 discount=0.00 tax=9.00 total=59.00"
   invoice("INV-1", lines)     "INVOICE #INV-1 net=50.00 discount=0.00 tax=9.00 total=59.00"
   credit_note("INV-1", lines) "CREDIT NOTE #INV-1 net=50.00 discount=0.00 tax=9.00 total=-59.00"
   ```

   `net` = sum of lines; `discount = rules.discount(net)`; `tax = rules.tax(net - discount)`; `total = net - discount + tax`. A credit note prints the total negated (net, discount and tax stay positive). Each method returns `Result<String, DocError>`.
3. Empty lines or any negative line gives `Err(DocError::InvalidLines)`; a blank invoice number (invoice and credit note) gives `Err(DocError::InvalidNumber)`; the same in all three documents. A single line of `0` is valid.
4. Money prints as `major.minor` with exactly two digits (`0.05`, `12.40`, `0.00`), with `-` in front when negative.

## Examples

With `StandardRules::new(1800, 10_000, 1000)` (18% tax; 10% off from 100.00):

| Call | Result |
|---|---|
| `quote(&[5000])` | `Ok("QUOTE net=50.00 discount=0.00 tax=9.00 total=59.00")` |
| `invoice("INV-1", &[12000, 8000])` | `Ok("INVOICE #INV-1 net=200.00 discount=20.00 tax=32.40 total=212.40")` (net 20000, discount 2000, tax 18% of 18000 = 3240) |
| `credit_note("INV-9", &[5000])` | `Ok("CREDIT NOTE #INV-9 net=50.00 discount=0.00 tax=9.00 total=-59.00")` |
| `quote(&[1005])` | `... net=10.05 discount=0.00 tax=1.81 total=11.86` (180.9 rounds up to 181) |
| `quote(&[5])` | `QUOTE net=0.05 discount=0.00 tax=0.01 total=0.06` |
| `quote(&[9999])` / `quote(&[10000])` | discount `0.00` / `10.00` (threshold is inclusive) |
| `quote(&[0])` | `QUOTE net=0.00 discount=0.00 tax=0.00 total=0.00` |

With custom rules `discount = 100`, `tax = 7` (always), `quote(&[1000])` gives `QUOTE net=10.00 discount=1.00 tax=0.07 total=9.07`, and the invoice and credit note show the same figures.

## Edge cases

- `quote(&[])`, `invoice("A", &[100, -1])` and `credit_note("A", &[-5])` give `Err(InvalidLines)`; `invoice(" ", &[100])` and `credit_note("", &[100])` give `Err(InvalidNumber)`.

## Running

```bash
cargo test -p p42_dry     # from rust/ — tests FAIL until you implement the stubs
cargo play run 42
```

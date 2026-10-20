# 42 · DRY — Don't Repeat Yourself (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 30 min | Level 2 — the `PricingRules` trait and `DocError` are provided; you write `StandardRules` and `Documents`. |

## Problem

A shop prints three documents: a **quote** (an estimate), an **invoice** (final) and a **credit note** (a refund).
They all use the same pricing and the same money formatting. Last quarter the tax rate changed and the credit notes
were still using the old one.

## Requirements

1. `StandardRules::new(tax_bps, bulk_threshold, bulk_discount_bps)`: `discount(net)` = `net * bulk_discount_bps / 10000` when `net >= threshold`, else 0; `tax(taxable)` = `taxable * tax_bps / 10000`. Both **round half up** to whole cents.
2. `Documents` formats the three documents exactly as shown in its doc comment. `total = net - discount + tax`, tax charged on the discounted amount; a credit note prints the total negated.
3. Empty lines or a negative line → `Err(DocError::InvalidLines)`; a blank invoice number → `Err(DocError::InvalidNumber)` — the same in all three documents.
4. Money prints as `major.minor` with two digits (`0.05`, `12.40`), with `-` in front when negative.

## Running

```bash
cargo test -p p42_dry     # from rust/ — tests FAIL until you implement the stubs
cargo play run 42
```

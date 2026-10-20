# 42 · DRY — Don't Repeat Yourself (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 30 min | Level 2 — the `PricingRules` interface is provided; you write `StandardRules` and `Documents`. |

## Problem

A shop prints three documents: a **quote** (an estimate), an **invoice** (final) and a **credit note** (a refund).
They all use the same pricing and the same money formatting. Last quarter the tax rate changed and the credit notes
were still using the old one.

## Requirements

1. `StandardRules(taxBps, bulkThresholdCents, bulkDiscountBps)`: `discount(net)` = `net * bulkDiscountBps / 10000` when `net >= threshold`, else 0; `tax(taxable)` = `taxable * taxBps / 10000`. Both **round half up** to whole cents.
2. `Documents` formats the three documents exactly as shown in its Javadoc. `total = net - discount + tax` where tax is charged on the discounted amount; a credit note prints the total negated.
3. Invalid input (null/empty lines, a negative line, a blank invoice number) is rejected with `IllegalArgumentException` — the same in all three documents.
4. Money prints as `major.minor` with two digits (`0.05`, `12.40`), with `-` in front when negative.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

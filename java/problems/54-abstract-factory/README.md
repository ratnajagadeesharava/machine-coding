# 54 · Abstract Factory (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — the product interfaces and `RegionKit` are provided; the rest is yours to write. |

## Problem

A shop sells in the US, the EU and India. Each region has its own tax rate, price format and postal-code rule. The
checkout summary is produced from a region kit.

## Requirements

| Region | `region()` | tax | price format | postal code |
|---|---|---|---|---|
| `UsKit` | `US` | 8% (800 bps) | `$12.50` | `12345` or `12345-6789` |
| `EuKit` | `EU` | 19% (1900 bps) | `12,50 EUR` | exactly 5 digits |
| `IndiaKit` | `IN` | 18% (1800 bps) | `Rs 12.50` | exactly 6 digits |

1. Tax is rounded **half up** to whole cents (`tax(1257)` at 8% is `101`). Negative amounts → `IllegalArgumentException` from both `tax` and `format`.
2. `isValidPostalCode(null)` is `false`.
3. `Kits.forRegion(code)`: `"US"`, `"EU"`, `"IN"` in any case; `null` or unknown → `IllegalArgumentException`.
4. `CheckoutSummary.summarize(kit, netCents, postalCode)` returns `"<region>: net <net>, tax <tax>, total <total>, postal <valid|invalid>"` — all amounts written by the kit's formatter, tax from the kit's calculator, `total = net + tax`. Example: `US: net $100.00, tax $8.00, total $108.00, postal valid`.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

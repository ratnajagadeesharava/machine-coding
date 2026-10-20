# 54 · Abstract Factory (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — the product interfaces and `RegionKit` are provided; the rest is yours to write. |

## Problem

**What you are building.** A shop sells in the US, the EU and India. Each region has its own tax rate, its own way of
writing prices, and its own postal-code rule. A *region kit* bundles those three behaviours for one region. The checkout
summary is produced from whatever kit it is handed, and must never mix behaviours from different regions (no US tax with an
EU price format). Amounts are in **cents** (`long`).

**Glossary**
- *Region kit* — a `RegionKit`: `region()` code plus `taxCalculator()`, `priceFormatter()`, `addressValidator()`.
- *bps (basis points)* — hundredths of a percent: 800 bps = 8%, 1900 bps = 19%, 1800 bps = 18%.
- *Net / tax / total* — amount before tax / tax due on it / `net + tax`.
- *Half up* — a fractional half cent rounds up (to the larger value).

## Requirements

Provided in `contract/`: `RegionKit`, `TaxCalculator.tax(netCents)`, `PriceFormatter.format(cents)`, `AddressValidator.isValidPostalCode(code)`. You write `UsKit`, `EuKit`, `IndiaKit`, `Kits`, `CheckoutSummary`.

| Region | `region()` | tax | price format | postal code |
|---|---|---|---|---|
| `UsKit` | `US` | 8% (800 bps) | `$12.50` | `12345` or `12345-6789` (5 digits, optionally a hyphen and 4 more digits) |
| `EuKit` | `EU` | 19% (1900 bps) | `12,50 EUR` | exactly 5 digits |
| `IndiaKit` | `IN` | 18% (1800 bps) | `Rs 12.50` | exactly 6 digits |

Price format details: always two decimal digits (`5` cents → `$0.05`, `0,05 EUR`); no thousands separator (`100000` cents → `$1000.00`); US and India use `.` as the decimal mark, the EU uses `,` and the suffix ` EUR`; India uses the prefix `Rs ` (with a space).

1. Tax is `netCents × rate`, rounded **half up** to whole cents (`tax(1257)` at 8% is `101` because 100.56 rounds up; `tax(1255)` at 8% is `100` because 100.4 rounds down). Negative amounts → `IllegalArgumentException` from both `tax` and `format`.
2. `isValidPostalCode(null)` is `false`; anything not matching the region's rule (wrong length, letters, surrounding text) is `false`.
3. `Kits.forRegion(code)`: `"US"`, `"EU"`, `"IN"` in any letter case (`"us"`, `"In"`) return the matching kit; `null` or any other code (e.g. `"JP"`) → `IllegalArgumentException`.
4. `CheckoutSummary.summarize(kit, netCents, postalCode)` returns `"<region>: net <net>, tax <tax>, total <total>, postal <valid|invalid>"` — `<region>` from `kit.region()`, all three amounts written by the kit's formatter, tax from the kit's calculator, `total = net + tax`, postal validity from the kit's validator. It must work with any `RegionKit`, including one it has never seen (e.g. a test kit with region `TEST`, tax always 1, format `#<cents>`, validator always false).

## Examples

| Call | Result |
|---|---|
| `summarize(new UsKit(), 10000, "12345")` | `US: net $100.00, tax $8.00, total $108.00, postal valid` |
| `summarize(new EuKit(), 10000, "10115")` | `EU: net 100,00 EUR, tax 19,00 EUR, total 119,00 EUR, postal valid` |
| `summarize(new IndiaKit(), 10000, "5600")` | `IN: net Rs 100.00, tax Rs 18.00, total Rs 118.00, postal invalid` |
| test kit (`TEST`, tax 1, `#cents`, never valid): `summarize(kit, 100, "whatever")` | `TEST: net #100, tax #1, total #101, postal invalid` |

Other values: US `format(1250)` = `$12.50`; EU `format(1250)` = `12,50 EUR`; India `format(1250)` = `Rs 12.50`. `tax(10000)` = `800` (US) / `1900` (EU) / `1800` (IN). Half-up check: EU `tax(50)` = `10` (9.5 rounds up); India `tax(25)` = `5` (4.5 rounds up). Postal: US `"1234"` and `"ABCDE"` invalid; EU `"12345-6789"` invalid; India `"12345"` invalid, `"560001"` valid.

## Edge cases

- Net of 0 is allowed (tax 0, price `$0.00`); only negatives are rejected.
- Repeated calls to `taxCalculator()` etc. must yield usable products every time.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

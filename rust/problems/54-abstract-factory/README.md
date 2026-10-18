# 54 · Abstract Factory (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 2 — the product traits and the `RegionKit` abstract factory trait are provided in `src/contract.rs`; three kits, a lookup and a client function are stubs. |

> **Pattern:** *Abstract Factory* — a trait for creating **families of related objects** without naming their concrete
> types. A kit produces the tax calculator, price formatter and address validator of one region, so a checkout can
> never combine US tax with EU number formatting.

## Problem

A shop sells in the US, the EU and India. Each region has its own tax rate, price format and postal-code rule. The
checkout summary code must not contain a single `match` on the region; it receives a kit and uses it.

## Requirements

| Region | `region()` | tax | price format | postal code |
|---|---|---|---|---|
| `UsKit` | `US` | 8% (800 bps) | `$12.50` | `12345` or `12345-6789` |
| `EuKit` | `EU` | 19% (1900 bps) | `12,50 EUR` | exactly 5 digits |
| `IndiaKit` | `IN` | 18% (1800 bps) | `Rs 12.50` | exactly 6 digits |

1. Tax is rounded **half up** to whole cents (`tax(1257)` at 8% is `101`).
2. `kit_for_region(code)`: `"US"`, `"EU"`, `"IN"` in any case; anything else → `Err(KitError::UnknownRegion)`.
3. `summarize(kit, net_cents, postal_code)` returns `"<region>: net <net>, tax <tax>, total <total>, postal <valid|invalid>"` — all amounts written by the kit's formatter, tax from the kit's calculator, `total = net + tax`. Example: `US: net $100.00, tax $8.00, total $108.00, postal valid`.

## Constraints (what the tests catch)

- The tests implement a whole `TestKit` family (three product types plus the kit) and pass it to `summarize`. A client that matches on `kit.region()` fails.
- **Rust adaptation:** the product types behind each kit are private types you design; the kits are unit structs.

## Running

```bash
cargo test -p p54_abstract_factory     # from rust/ — tests FAIL until you implement the stubs
cargo play run 54
```

# 66 · Strategy (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — `Parcel`, `ShippingStrategy` and `ShipError` provided; write the four strategies, `ShippingCalculator` and `cheapest`. |

## Problem

A web shop prices delivery in several ways and marketing changes the rule every season: flat rate, by weight, by
distance, "free over €100". Each rule is a strategy; the checkout asks the current one for a price.

**What you are building:** the shipping-price engine of a web shop. There are several pricing rules (flat rate,
per kilogram, per kilometre, and "free if the order is big enough, otherwise use another rule"). A
`ShippingCalculator` quotes a parcel using whichever rule is currently selected, and the rule can be swapped while
the program runs. The free function `cheapest` looks at a list of rules and tells you which is cheapest for a given
parcel.

### Provided types

- `Parcel { weight_grams: u32, distance_km: u32, value_cents: u64 }` — what is being shipped.
- `ShippingStrategy` — `fn name(&self) -> String` (a short label) and `fn cost(&self, parcel: &Parcel) -> u64` (the price in cents). Callers (and tests) may supply their own implementations, e.g. a `"pickup"` rule that always costs 0.
- `ShipError::{InvalidParcel, NoStrategies}` — `InvalidParcel` is a parcel under 1 gram; `NoStrategies` is `cheapest` called with an empty list.

### Glossary

- **Cents**: all money values are whole cents (`499` = 4.99). Parcel value is also in cents, so "€100" is `10_000`.
- **Started kilogram**: weight is rounded **up** to whole kilograms.
- **Wrapping**: one strategy holding another and delegating to it.

## Requirements

1. `FlatRate::new(cents)` — `name()` `"flat"`; costs `cents` for every parcel. `ByWeight::new(cents_per_kg)` — `"weight"`, billed per **started** kilogram (1 g and 1000 g are 1 kg, 1001 g is 2 kg); cost = started kilograms × `cents_per_kg`. `DistanceBased::new(cents_per_km)` — `"distance"`; cost = `distance_km` × `cents_per_km` (0 km costs 0).
2. `FreeOverThreshold::new(min_value_cents, fallback)` — cost is 0 when `parcel.value_cents >= min` (exactly equal counts as free), otherwise the fallback's cost for that parcel. `name()` is `"free>=<min>/<fallback name>"`, e.g. `"free>=10000/flat"`. Strategies can wrap strategies, so the name nests: `"free>=50000/free>=10000/weight"`.
3. `ShippingCalculator::new(Box<dyn ShippingStrategy>)`: `quote(&parcel)` returns `Ok(cost)` from the current strategy; `set_strategy` swaps it at runtime (later quotes use the new one); `strategy_name()` returns the current strategy's name. A parcel under 1 gram → `Err(ShipError::InvalidParcel)`. Distance 0 and value 0 are valid.
4. `cheapest(&[Box<dyn ShippingStrategy>], &parcel)` returns `Ok(index)` of the strategy with the lowest `cost(parcel)` (on a tie the earlier index wins). Empty list → `Err(NoStrategies)`; parcel under 1 gram → `Err(InvalidParcel)`.

## Examples

Sample parcels used below: `LIGHT_NEAR = {500 g, 10 km, 2000 ¢}`, `HEAVY_FAR = {2500 g, 300 km, 20000 ¢}`.

| Strategy | Parcel | `cost` |
|---|---|---|
| `FlatRate::new(499)` | `LIGHT_NEAR` or `HEAVY_FAR` | 499 |
| `ByWeight::new(200)` | 1 g / 1000 g / 1001 g | 200 / 200 / 400 |
| `ByWeight::new(200)` | `HEAVY_FAR` (3 started kg) | 600 |
| `DistanceBased::new(3)` | `LIGHT_NEAR` / `HEAVY_FAR` / 0 km | 30 / 900 / 0 |
| `FreeOverThreshold::new(10000, FlatRate(499))` | value 10000 / value 9999 | 0 / 499 |
| `FreeOverThreshold::new(50000, FreeOverThreshold::new(10000, ByWeight(100)))` | 2500 g with value 60000 / 20000 / 5000 | 0 / 0 / 300 |

`cheapest`, options `[FlatRate(499), ByWeight(200), DistanceBased(3), Pickup]` (indexes 0..3):

- `HEAVY_FAR`: costs 499, 600, 900, 0 → `Ok(3)`.
- First three options only, parcel `LIGHT_NEAR`: costs 499, 200, 30 → `Ok(2)`.
- `[FlatRate(100), ByWeight(100)]` with `{1000 g, 1 km, 1 ¢}`: both cost 100 → `Ok(0)` (earlier wins).

Calculator: `ShippingCalculator::new(Box::new(FlatRate::new(499))).quote(&HEAVY_FAR)` is `Ok(499)`; after `set_strategy(Box::new(DistanceBased::new(3)))` it is `Ok(900)` and `strategy_name()` is `"distance"`.

## Edge cases

- `quote(&Parcel { weight_grams: 0, distance_km: 1, value_cents: 1 })` → `Err(InvalidParcel)`.
- `cheapest(&[], &parcel)` → `Err(NoStrategies)`; `cheapest` with a valid list but a 0 g parcel → `Err(InvalidParcel)`.

## Running

```bash
cargo test -p p66_strategy     # from rust/ — tests FAIL until you implement the stubs
cargo play run 66
```

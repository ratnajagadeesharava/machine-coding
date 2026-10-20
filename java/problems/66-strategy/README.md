# 66 · Strategy (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — `Parcel` and `ShippingStrategy` provided; write the four strategies, `ShippingCalculator` and `ShippingAdvisor`. |

## Problem

A web shop prices delivery in several ways and marketing changes the rule every season: flat rate, by weight, by
distance, "free over €100". Each rule is a strategy; the checkout asks the current one for a price.

**What you are building:** the shipping-price engine of a web shop. There are several pricing rules (flat rate,
per kilogram, per kilometre, and "free if the order is big enough, otherwise use another rule"). A
`ShippingCalculator` quotes a parcel using whichever rule is currently selected, and the rule can be swapped while
the program runs. A `ShippingAdvisor` looks at a list of rules and tells you which is cheapest for a given parcel.

### Provided types

- `Parcel(int weightGrams, int distanceKm, long valueCents)` — what is being shipped.
- `ShippingStrategy` — `String name()` (a short label) and `long cost(Parcel)` (the price in cents). Callers (and tests) may supply their own implementations, e.g. a `"pickup"` rule that always costs 0.

### Glossary

- **Cents**: all money values are whole cents (`499` = 4.99). Parcel value is also in cents, so "€100" is `10_000`.
- **Started kilogram**: weight is rounded **up** to whole kilograms.
- **Wrapping**: one strategy holding another and delegating to it.

## Requirements

1. `FlatRate(cents)` — `name()` `"flat"`; costs `cents` for every parcel. `ByWeight(centsPerKg)` — `"weight"`, billed per **started** kilogram (1 g and 1000 g are 1 kg, 1001 g is 2 kg); cost = started kilograms × `centsPerKg`. `DistanceBased(centsPerKm)` — `"distance"`; cost = `distanceKm` × `centsPerKm` (0 km costs 0). Negative prices → `IllegalArgumentException` (thrown by the constructor).
2. `FreeOverThreshold(minValueCents, fallback)` — cost is 0 when `parcel.valueCents() >= min` (exactly equal counts as free), otherwise the fallback's cost for that parcel. `name()` is `"free>=<min>/<fallback name>"`, e.g. `"free>=10000/flat"`. Strategies can wrap strategies, so the name nests: `"free>=50000/free>=10000/weight"`. Negative minimum or `null` fallback → `IllegalArgumentException`.
3. `ShippingCalculator(strategy)`: `quote(parcel)` returns the current strategy's `cost`; `setStrategy` swaps it at runtime (later quotes use the new one); `strategy()` returns the current one. `null` strategy (constructor or `setStrategy`) → `IllegalArgumentException`. A `null` parcel, weight below 1 g, negative distance or negative value → `IllegalArgumentException` from `quote`. A distance of 0 and a value of 0 are valid.
4. `ShippingAdvisor.cheapest(strategies, parcel)` returns the strategy with the lowest `cost(parcel)` (on a tie the one earlier in the list wins); `null`/empty list or an invalid parcel (same rules as `quote`) → `IllegalArgumentException`.

## Examples

Sample parcels used below: `LIGHT_NEAR = Parcel(500 g, 10 km, 2000 ¢)`, `HEAVY_FAR = Parcel(2500 g, 300 km, 20000 ¢)`.

| Strategy | Parcel | `cost` |
|---|---|---|
| `FlatRate(499)` | `LIGHT_NEAR` or `HEAVY_FAR` | 499 |
| `ByWeight(200)` | 1 g / 1000 g / 1001 g | 200 / 200 / 400 |
| `ByWeight(200)` | `HEAVY_FAR` (3 started kg) | 600 |
| `DistanceBased(3)` | `LIGHT_NEAR` / `HEAVY_FAR` / 0 km | 30 / 900 / 0 |
| `FreeOverThreshold(10000, FlatRate(499))` | value 10000 / value 9999 | 0 / 499 |
| `FreeOverThreshold(50000, FreeOverThreshold(10000, ByWeight(100)))` | 2500 g with value 60000 / 20000 / 5000 | 0 / 0 / 300 |

Advisor, options `[FlatRate(499), ByWeight(200), DistanceBased(3), Pickup]`:

- `HEAVY_FAR`: costs 499, 600, 900, 0 → `pickup`.
- Same list without `Pickup`, parcel `LIGHT_NEAR`: costs 499, 200, 30 → `distance`.
- `[FlatRate(100), ByWeight(100)]` with `Parcel(1000, 1, 1)`: both cost 100 → `flat` (earlier wins).

Calculator: `new ShippingCalculator(new FlatRate(499)).quote(HEAVY_FAR)` is 499; after `setStrategy(new DistanceBased(3))` it is 900 and `strategy().name()` is `"distance"`.

## Edge cases

- `quote(new Parcel(0, 1, 1))`, `quote(new Parcel(1, -1, 1))`, `quote(new Parcel(1, 1, -1))` and `quote(null)` all throw `IllegalArgumentException`.
- `new FlatRate(-1)`, `new FreeOverThreshold(100, null)` throw `IllegalArgumentException`.
- `cheapest(List.of(), parcel)` throws `IllegalArgumentException`.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

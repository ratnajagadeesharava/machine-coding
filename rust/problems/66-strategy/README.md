# 66 · Strategy (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — `Parcel`, `ShippingStrategy` and `ShipError` provided; write the four strategies, `ShippingCalculator` and `cheapest`. |

## Problem

A web shop prices delivery in several ways and marketing changes the rule every season: flat rate, by weight, by
distance, "free over €100". Each rule is a strategy; the checkout asks the current one for a price.

## Requirements

1. `FlatRate::new(cents)` — `name()` `"flat"`. `ByWeight::new(cents_per_kg)` — `"weight"`, billed per **started** kilogram (1 g and 1000 g are 1 kg, 1001 g is 2 kg). `DistanceBased::new(cents_per_km)` — `"distance"`.
2. `FreeOverThreshold::new(min_value_cents, fallback)` — free when `parcel.value_cents >= min`, otherwise the fallback's cost. `name()` is `"free>=<min>/<fallback name>"`. Strategies can wrap strategies.
3. `ShippingCalculator::new(Box<dyn ShippingStrategy>)`: `quote(&parcel)` uses the current strategy; `set_strategy` swaps it at runtime; `strategy_name()` names it. A parcel under 1 gram → `Err(ShipError::InvalidParcel)`.
4. `cheapest(&[Box<dyn ShippingStrategy>], &parcel)` returns the **index** of the cheapest strategy (first wins a tie). Empty list → `Err(NoStrategies)`; parcel under 1 gram → `Err(InvalidParcel)`.

## Running

```bash
cargo test -p p66_strategy     # from rust/ — tests FAIL until you implement the stubs
cargo play run 66
```

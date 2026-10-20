# 66 · Strategy (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — `Parcel` and `ShippingStrategy` provided; write the four strategies, `ShippingCalculator` and `ShippingAdvisor`. |

## Problem

A web shop prices delivery in several ways and marketing changes the rule every season: flat rate, by weight, by
distance, "free over €100". Each rule is a strategy; the checkout asks the current one for a price.

## Requirements

1. `FlatRate(cents)` — `name()` `"flat"`. `ByWeight(centsPerKg)` — `"weight"`, billed per **started** kilogram (1 g and 1000 g are 1 kg, 1001 g is 2 kg). `DistanceBased(centsPerKm)` — `"distance"`. Negative prices → `IllegalArgumentException`.
2. `FreeOverThreshold(minValueCents, fallback)` — free when `parcel.valueCents() >= min`, otherwise the fallback's cost. `name()` is `"free>=<min>/<fallback name>"`. Strategies can wrap strategies. Negative minimum or `null` fallback → `IllegalArgumentException`.
3. `ShippingCalculator(strategy)`: `quote(parcel)` uses the current strategy; `setStrategy` swaps it at runtime; `strategy()` returns it. `null` strategy → `IllegalArgumentException`. A `null` parcel, weight below 1 g, negative distance or negative value → `IllegalArgumentException`.
4. `ShippingAdvisor.cheapest(strategies, parcel)` returns the cheapest strategy (first one wins a tie); `null`/empty list or an invalid parcel → `IllegalArgumentException`.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

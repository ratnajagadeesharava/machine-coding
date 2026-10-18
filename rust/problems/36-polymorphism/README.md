# 36 · Polymorphism (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 30 min | Level 2 — the `Vehicle` trait and `TollError` are provided in `src/contract.rs`; the vehicle types and `TollPlaza` are stubs with `todo!()`. |

> **Concept:** one call, many behaviours. The caller invokes `vehicle.toll(peak)` through `&dyn Vehicle` and the
> *concrete type* decides the answer. No `match` on a type tag, no downcasting.

## Problem

A highway toll plaza charges different vehicles different tolls, with a rush-hour surcharge that is not the same
for every vehicle. New vehicle types appear every year (ambulances, limousines...).

## Requirements

1. `contract::Vehicle` (provided): `plate()` and `toll(peak)` in cents.
2. `Car`: 500, peak 750. `Motorcycle`: 200, never surcharged. `Truck::new(plate, axles)`: 1000 + 300 per axle beyond 2, peak = 1.5 × that (fewer than 2 axles → `Err(TollError::InvalidAxles)`). `Bus`: 800, peak 960.
3. `TollPlaza::charge(&dyn Vehicle, peak)` returns the vehicle's own toll and records a receipt `"PLATE:amount"`.
4. `total()` is the sum of all charges; `receipts()` are in charge order.

## Constraints

- `TollPlaza` takes `&dyn Vehicle` and must not know which concrete types exist. The tests pass in types defined only in the test file (`Ambulance`, `Limo`).
- Integer arithmetic only.

## Running

```bash
cargo test -p p36_polymorphism     # from rust/ — tests FAIL until you implement the stubs
cargo play run 36
```

# 36 · Polymorphism (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 30 min | Level 2 — the `Vehicle` interface is provided; four vehicle classes and `TollPlaza` are stubs. |

> **Concept:** one call, many behaviours. The caller invokes `vehicle.toll(peak)` and the *runtime type* decides
> the answer. No `instanceof` chains, no `switch` on a type tag.

## Problem

A highway toll plaza charges different vehicles different tolls, with a rush-hour surcharge that is not the same
for every vehicle. New vehicle types appear every year (ambulances, limousines...).

## Requirements

1. `contract.Vehicle` (provided): `plate()` and `toll(boolean peak)` in cents.
2. `Car`: 500, peak 750. `Motorcycle`: 200, never surcharged. `Truck(plate, axles)`: 1000 + 300 per axle beyond 2, peak = 1.5 × that (axles below 2 → `IllegalArgumentException`). `Bus`: 800, peak 960.
3. `TollPlaza.charge(vehicle, peak)` returns the vehicle's own toll and records a receipt `"PLATE:amount"`. `null` → `IllegalArgumentException` and nothing is recorded.
4. `total()` is the sum of all charges; `receipts()` are in charge order and cannot be modified by callers.

## Constraints

- `TollPlaza` must not use `instanceof`, `getClass()` or a type switch. The tests pass in vehicle classes that exist only in the test file (`Ambulance`, `Limo`).
- Use integer arithmetic (no `double`).

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

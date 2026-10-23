# 47 · OCP — Open/Closed Principle (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — the `FeePolicy` interface is provided; the rest is yours to write. |

## Problem

A parking garage prices stays by vehicle type. Today it handles cars and motorcycles. Next month: electric vehicles.
After that: boats (it is a harbour garage).

## Requirements

1. `CarPolicy` — supports `"car"` (any case): 0 minutes → 0; otherwise **200** for the first 60 minutes plus **100 per started 30 minutes** after that, capped at **1500**. (30→200, 60→200, 61→300, 90→300, 91→400, 1000→1500.)
2. `MotorcyclePolicy` — supports `"motorcycle"`: half of the car fee (integer division).
3. `FeeCalculator(List<FeePolicy>)` + `register(policy)`: `fee(type, minutes)` uses the **most recently registered** policy that supports the type. No policy for the type, `null` type or negative minutes → `IllegalArgumentException`.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

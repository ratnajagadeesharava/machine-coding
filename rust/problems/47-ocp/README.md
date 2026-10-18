# 47 · OCP — Open/Closed Principle (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — the `FeePolicy` trait and `FeeError` are provided; the two policies (`src/policies.rs`) and the calculator (`src/calculator.rs`) are stubs. |

> **Principle:** software entities should be *open for extension, closed for modification*. Adding a new kind of
> vehicle should mean **adding** a type that implements `FeePolicy`, not editing a `match` in code that already works.

## Problem

A parking garage prices stays by vehicle type. Today it handles cars and motorcycles. Next month: electric vehicles.
After that: boats (it is a harbour garage). Build the pricing so none of those requires touching the calculator.

## Requirements

1. `CarPolicy` — supports `"car"` (any case): 0 minutes → 0; otherwise **200** for the first 60 minutes plus **100 per started 30 minutes** after that, capped at **1500**. (30→200, 60→200, 61→300, 90→300, 91→400, 1000→1500.)
2. `MotorcyclePolicy` — supports `"motorcycle"`: half of the car fee (integer division).
3. `FeeCalculator::new(Vec<Box<dyn FeePolicy>>)` + `register(policy)`: `fee(type, minutes)` uses the **most recently registered** policy that supports the type; none → `Err(FeeError::UnknownVehicle)`.

## Constraints (what the tests catch)

- The tests define `ElectricPolicy` and flat policies **inside the test file** and plug them in. They must work with zero changes to your code.
- A calculator built with only a `"boat"` policy must price boats and reject cars: it has no built-in knowledge.
- `src/calculator.rs` must not contain the strings `"car"` or `"motorcycle"`. Keep the three source files where they are.

## Running

```bash
cargo test -p p47_ocp     # from rust/ — tests FAIL until you implement the stubs
cargo play run 47
```

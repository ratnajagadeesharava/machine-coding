# 47 · OCP — Open/Closed Principle (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — the `FeePolicy` interface is provided; the rest is yours to write. |

## Problem

**What you are building.** A harbour parking garage prices a stay by vehicle type and how long it lasted. Today it handles cars and motorcycles; next month electric vehicles will have their own rules, and after that boats. You build the pricing so that each vehicle type has its own fee rule, and a calculator that finds the right rule for a vehicle and returns the fee. Adding a new vehicle type later (the tests add an electric vehicle and a boat) must work by supplying a new rule, without changing the calculator's behaviour for the existing ones.

## Requirements

Provided: interface `FeePolicy` with `boolean supports(String vehicleType)` (case-insensitive match on the vehicle type) and `long fee(int minutes)` (fee in cents for a stay of `minutes >= 0`). You write `CarPolicy`, `MotorcyclePolicy` and `FeeCalculator`. All fees are whole cents.

1. `CarPolicy` supports `"car"` (any letter case). Fee for a stay of `m` minutes:
   - `m == 0` → 0.
   - Otherwise **200** covers the first 60 minutes, plus **100 for every started 30 minutes** after minute 60, with the total **capped at 1500**.
   - Table: 1→200, 30→200, 60→200, 61→300, 90→300, 91→400, 120→400, 121→500, 450 or more→1500 (e.g. 1000→1500).
2. `MotorcyclePolicy` supports `"motorcycle"` (any letter case): half of the car fee for the same minutes, by integer division (30→100, 91→200, 1000→750).
3. `new FeeCalculator(List<FeePolicy> policies)` and `register(policy)`: `long fee(String vehicleType, int minutes)` uses the **most recently registered** policy that supports the type (policies given to the constructor count as registered first, in list order; anything passed to `register` later beats them). The calculator works with any starting list, including an immutable one such as `List.of(...)` or an empty list, and `register` still works. These cases throw `IllegalArgumentException` (and change nothing): no registered policy supports the type, a `null` type, or negative `minutes`.

## Examples

Car fee by stay length:

| Minutes | 0 | 30 | 60 | 61 | 90 | 91 | 120 | 1000 |
|---|---|---|---|---|---|---|---|---|
| Car | 0 | 200 | 200 | 300 | 300 | 400 | 400 | 1500 |
| Motorcycle | 0 | 100 | 100 | 150 | 150 | 200 | 200 | 750 |

Calculator:

```
calc = new FeeCalculator(List.of(new CarPolicy(), new MotorcyclePolicy()))
calc.fee("car", 0)          -> 0
calc.fee("CAR", 90)         -> 300      // type match ignores case
calc.fee("motorcycle", 91)  -> 200      // half of the car fee, 400 / 2
calc.fee("motorcycle", 1000)-> 750      // half of the cap, 1500 / 2
calc.fee("boat", 10)        -> IllegalArgumentException   // nobody supports "boat" yet
calc.register(boatPolicyThatAlwaysCosts999)
calc.fee("boat", 10)        -> 999      // new type added without touching the calculator
calc.register(freePolicyForCars)        // supports "car", fee always 0
calc.fee("car", 500)        -> 0        // most recently registered wins
calc.fee("motorcycle", 30)  -> 100      // unaffected
calc.fee("car", -1) / calc.fee(null, 10) -> IllegalArgumentException
```

A custom electric rule used in the tests (free for the first 60 minutes, then 50 per started 30 minutes) gives: 45 min → 0, 61 min → 50, 120 min → 100, with `"car"` for 61 min still 300.

## Edge cases

- Exactly 60 minutes still costs the base 200; the first extra minute (61) starts the next 30-minute block and costs 300.
- 0 minutes is free for cars and motorcycles.
- A calculator built with an empty list or with only custom policies (say, only a boat policy) works; `"car"` is then unknown and throws.
- Vehicle-type matching is the policy's own `supports` decision (the provided policies ignore case); the calculator asks each policy.
- Negative minutes are rejected even for a supported type.

## Glossary

- **Policy**: one vehicle type's fee rule: which type it supports and how much a stay of a given length costs.
- **Started 30 minutes**: any partial block counts as a full one (1 to 30 minutes past the hour costs one block, 31 to 60 costs two).
- **Cap**: the highest fee a policy will ever return.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

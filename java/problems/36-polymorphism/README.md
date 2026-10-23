# 36 · Polymorphism (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 30 min | Level 2 — the `Vehicle` interface is provided; four vehicle classes and `TollPlaza` are stubs. |

## Problem

You are building the charging logic of a highway toll plaza. Cars, motorcycles, trucks and buses each pay a different
toll, and during rush hour ("peak") the surcharge is not the same for every vehicle. The plaza receives a stream of
vehicles, charges each one the right amount through a single call, and keeps receipts so the operator can reconcile the
day's takings. New vehicle types appear every year (ambulances, limousines...), and the plaza must be able to charge a
vehicle type it has never seen before.

## Glossary

- **cents**: all amounts are whole numbers of cents (`500` = 5.00).
- **peak**: rush hour; `peak == true` means the surcharge applies where the vehicle has one.
- **receipt**: one string per charge, formatted `"PLATE:amount"`, e.g. `"AB-12:500"`.

## Requirements

1. `contract.Vehicle` (provided): `String plate()` and `long toll(boolean peak)`, the toll in cents.
2. Tolls, in cents:

   | Vehicle | Off-peak | Peak |
   |---|---|---|
   | `Car(plate)` | 500 | 750 |
   | `Motorcycle(plate)` | 200 | 200 (never surcharged) |
   | `Truck(plate, axles)` | 1000 + 300 per axle beyond 2 | 1.5 x the off-peak toll |
   | `Bus(plate)` | 800 | 960 |

   A `Truck` with fewer than 2 axles cannot be constructed: the constructor throws `IllegalArgumentException`.
   Each vehicle's `plate()` returns the plate it was constructed with.
3. `TollPlaza.charge(vehicle, peak)` returns the vehicle's own `toll(peak)` and records the receipt `"PLATE:amount"`
   (for example `"BUS-9:960"`). It must work for any `Vehicle`, including ones defined outside your code (the tests
   define an `Ambulance` that always pays 0 and a `Limo` that pays 2500, or 5000 at peak). A vehicle with toll 0 still
   gets a receipt (`"AMB:0"`). A `null` vehicle throws `IllegalArgumentException` and nothing is recorded.
4. `total()` is the sum of all amounts charged so far (0 for a new plaza). `receipts()` lists the receipts in charge
   order, and callers cannot modify the returned list (attempting to throws `UnsupportedOperationException`).

## Examples

| Call | Result |
|---|---|
| `new Truck("T", 2).toll(false)` | `1000` |
| `new Truck("T", 5).toll(false)` | `1900` (1000 + 3 x 300) |
| `new Truck("T", 4).toll(true)` | `2400` (1600 x 1.5) |
| `new Truck("T", 1)` | throws `IllegalArgumentException` |

```java
TollPlaza plaza = new TollPlaza();
plaza.charge(new Car("AB-12"), false);   // 500
plaza.charge(new Ambulance("AMB"), false); // 0
plaza.charge(new Bus("BUS-9"), true);    // 960
plaza.total();     // 1460
plaza.receipts();  // ["AB-12:500", "AMB:0", "BUS-9:960"]
plaza.receipts().add("x"); // UnsupportedOperationException
plaza.charge(null, false); // IllegalArgumentException, total() stays 1460
```

## Edge cases

- A truck with exactly 2 axles pays the base 1000 (no per-axle extra).
- The same plate may be charged more than once; each charge is its own receipt.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

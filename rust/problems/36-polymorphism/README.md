# 36 · Polymorphism (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Easy | 30 min | Level 2 — the `Vehicle` trait and `TollError` are provided in `src/contract.rs`; the vehicle types and `TollPlaza` are stubs with `todo!()`. |

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

1. `contract::Vehicle` (provided): `fn plate(&self) -> &str` and `fn toll(&self, peak: bool) -> u64`, the toll in cents.
2. Tolls, in cents:

   | Vehicle | Off-peak | Peak |
   |---|---|---|
   | `Car::new(plate)` | 500 | 750 |
   | `Motorcycle::new(plate)` | 200 | 200 (never surcharged) |
   | `Truck::new(plate, axles)` | 1000 + 300 per axle beyond 2 | 1.5 x the off-peak toll |
   | `Bus::new(plate)` | 800 | 960 |

   `Truck::new` returns a `Result`: fewer than 2 axles gives `Err(TollError::InvalidAxles)`. Each vehicle's `plate()`
   returns the plate it was constructed with.
3. `TollPlaza::charge(&mut self, &dyn Vehicle, peak)` returns the vehicle's own `toll(peak)` and records the receipt
   `"PLATE:amount"` (for example `"BUS-9:960"`). It must work for any type implementing `Vehicle`, including ones
   defined outside your code (the tests define an `Ambulance` that always pays 0 and a `Limo` that pays 2500, or 5000
   at peak). A vehicle with toll 0 still gets a receipt (`"AMB:0"`).
4. `total()` is the sum of all amounts charged so far (0 for `TollPlaza::new()`). `receipts()` lists the receipts in
   charge order (empty for a new plaza).

## Examples

| Call | Result |
|---|---|
| `Truck::new("T", 2).unwrap().toll(false)` | `1000` |
| `Truck::new("T", 5).unwrap().toll(false)` | `1900` (1000 + 3 x 300) |
| `Truck::new("T", 4).unwrap().toll(true)` | `2400` (1600 x 1.5) |
| `Truck::new("T", 1)` | `Err(TollError::InvalidAxles)` |

```rust
let mut plaza = TollPlaza::new();
plaza.charge(&Car::new("AB-12"), false);      // 500
plaza.charge(&Ambulance("AMB".into()), false); // 0
plaza.charge(&Bus::new("BUS-9"), true);       // 960
plaza.total();     // 1460
plaza.receipts();  // ["AB-12:500", "AMB:0", "BUS-9:960"]
```

## Edge cases

- A truck with exactly 2 axles pays the base 1000 (no per-axle extra).
- The same plate may be charged more than once; each charge is its own receipt.

## Running

```bash
cargo test -p p36_polymorphism     # from rust/ — tests FAIL until you implement the stubs
cargo play run 36
```

# 04 · Coffee Machine (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 60 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

**What you are building.** The software inside a multi-outlet coffee machine, like the ones in an office kitchen. Nothing is hard-coded: an operator teaches the machine **recipes** ("latte = 30 units water + 2 coffee + 5 milk") and tops up the **ingredient** tanks at any time. Users then ask for a drink by name at any of several outlets, possibly at the same moment. A drink is served only if every ingredient is sufficient; otherwise the user is told which ingredient is short and the stock is left untouched. Operators can also ask which ingredients are running low so they know what to refill.

## Requirements

1. Ingredients are named (`water`, `milk`, `coffee`, …) and tracked as integer unit counts. An ingredient that was never refilled has stock 0.
2. A **recipe** is a drink name plus the units of each ingredient it needs. Operators can add recipes at runtime; adding a recipe with an existing name replaces it.
3. Operators **refill** ingredients (`refill(ingredient, units)`); refills accumulate (5 then 7 gives 12).
4. A user can **dispense** a drink. It succeeds only if *every* ingredient is sufficient (stock `>=` the recipe quantity, so stock may drop exactly to 0). A successful dispense consumes the recipe's quantities; a failed one consumes **nothing**, not even the ingredients that were sufficient.
5. When a dispense fails because of stock, the result names the missing ingredient (the alphabetically first one if several are short).
6. Operators can list the ingredients that are **running low** (stock strictly below a threshold).
7. The machine has several **outlets** that may serve users at the same time. Stock must never go negative, and no ingredient may be consumed for a drink that was refused.

## Constraints

- Any number of recipes and ingredients; recipe size is small (< 20 ingredients).
- Concurrent dispenses are normal: dispensing different drinks should not needlessly serialize unrelated work, but correctness is non-negotiable.
- A recipe must have at least one ingredient and every quantity must be positive, otherwise it is rejected.

## Examples

**1. Success, then shortage** (the machine starts empty)

```
refill("water", 100); refill("coffee", 10); refill("milk", 5)
add_recipe("latte", {water: 30, coffee: 2, milk: 5})  -> true
dispense("latte") -> "OK"                  stock: water 70, coffee 8, milk 0
dispense("latte") -> "INSUFFICIENT:milk"   stock unchanged: water 70, coffee 8, milk 0
dispense("tea")   -> "UNKNOWN_DRINK"
```

**2. Nothing is consumed on failure**

```
refill("water", 100); refill("coffee", 10); refill("milk", 2)
add_recipe("latte", {water: 30, coffee: 2, milk: 5})
dispense("latte") -> "INSUFFICIENT:milk"
stock: water 100, coffee 10, milk 2          (water and coffee were enough, still untouched)
```

**3. Replacing a recipe and invalid recipes**

```
refill("water", 100); refill("coffee", 100)
add_recipe("mix", {water: 10}) -> true
add_recipe("mix", {coffee: 4}) -> true        (replaces the earlier recipe)
dispense("mix") -> "OK"                      water 100, coffee 96
add_recipe("nothing", {})          -> false   (no ingredients)
add_recipe("bad", {water: 0})      -> false   (quantity must be positive)
add_recipe("worse", {water: -3})   -> false
dispense("bad") -> "UNKNOWN_DRINK"           (rejected recipes are not stored)
```

**4. Low stock** (water 100, milk 3, coffee 4, sugar 10)

| Call | Result |
|---|---|
| `low_stock(5)` | `["coffee", "milk"]` |
| `low_stock(4)` | `["milk"]` (4 is not strictly below 4) |
| `low_stock(1)` | `[]` |

**5. Several outlets racing** (milk 50, recipe "latte" = 5 milk, 40 simultaneous `dispense("latte")`): exactly 10 return `OK`, exactly 30 return `INSUFFICIENT:milk`, and final milk stock is 0.

## Edge cases

- Two drinks that share an ingredient compete for it: with water 50 and two recipes of 30 water each, the first dispense is `OK` and the second is `INSUFFICIENT:water`.
- Several ingredients short at once: the reported name is the alphabetically first short one (e.g. short on both `milk` and `coffee` gives `INSUFFICIENT:coffee`).
- Consuming an ingredient down to 0 keeps it known, so `low_stock` still lists it (0 is below any positive threshold).
- Under concurrent mixed drinks, final stock always equals the starting stock minus what the successful dispenses consumed, and never goes negative.

## Glossary

- **Ingredient**: a named consumable with an integer unit count.
- **Recipe**: a drink name mapped to the units of each ingredient it needs.
- **Outlet**: one place a user can request a drink from; several may be used at the same time.
- **Low stock**: an ingredient whose current units are strictly less than the threshold you pass.

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `dispense` returns exactly `OK`, `UNKNOWN_DRINK`, or `INSUFFICIENT:<ingredient>` (first missing ingredient in alphabetical order; an ingredient never refilled has stock 0).
- `add_recipe` returns `false` (and changes nothing) for an empty ingredient set or any quantity ≤ 0; `true` otherwise.
- `stock(ingredient)` is 0 for unknown ingredients.
- `low_stock(threshold)` lists ingredients whose stock is **strictly below** the threshold, sorted alphabetically. It includes ingredients that were refilled and then fully consumed.
- All operations must be safe to call concurrently from multiple threads.
- The drink name in `dispense` is matched exactly (case-sensitive) against recipe names.
- When several ingredients are short, the single name returned in `INSUFFICIENT:<ingredient>` is the alphabetically smallest short ingredient.
- Replacing a recipe affects later dispenses only; stock already consumed is not refunded.

## Test entry point (Rust)

The tests drive your system through the thin, primitive-typed contract below (`src/contract.rs`). It is
**not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up
in `src/lib.rs`.

```rust
//! Thin test entry point. Your domain model lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

pub trait CoffeeMachineApi: Send + Sync {
    fn refill(&self, ingredient: &str, units: u32);

    /// Defines or replaces a recipe. Returns false if the recipe is invalid.
    fn add_recipe(&self, drink: &str, ingredients: &[(&str, u32)]) -> bool;

    /// "OK", "UNKNOWN_DRINK" or "INSUFFICIENT:<ingredient>".
    fn dispense(&self, drink: &str) -> String;

    fn stock(&self, ingredient: &str) -> u32;

    /// Ingredients with stock strictly below the threshold, sorted alphabetically.
    fn low_stock(&self, threshold: u32) -> Vec<String>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create() -> Option<Arc<dyn CoffeeMachineApi>> {
    None
}
```

## Language notes

The trait is `&self` + `Send + Sync`: choose where the interior mutability lives (one `Mutex`, per-ingredient locks, atomics?) and defend it.

## Running

```bash
cargo test -p p04_coffee_machine
STRICT=1 cargo test -p p04_coffee_machine   # fail instead of skip while unimplemented
cargo test -p p04_coffee_machine --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p04_coffee_machine --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

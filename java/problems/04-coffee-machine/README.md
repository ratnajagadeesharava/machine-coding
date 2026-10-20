# 04 · Coffee Machine (Java)

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
addRecipe("latte", {water: 30, coffee: 2, milk: 5})  -> true
dispense("latte") -> "OK"                  stock: water 70, coffee 8, milk 0
dispense("latte") -> "INSUFFICIENT:milk"   stock unchanged: water 70, coffee 8, milk 0
dispense("tea")   -> "UNKNOWN_DRINK"
```

**2. Nothing is consumed on failure**

```
refill("water", 100); refill("coffee", 10); refill("milk", 2)
addRecipe("latte", {water: 30, coffee: 2, milk: 5})
dispense("latte") -> "INSUFFICIENT:milk"
stock: water 100, coffee 10, milk 2          (water and coffee were enough, still untouched)
```

**3. Replacing a recipe and invalid recipes**

```
refill("water", 100); refill("coffee", 100)
addRecipe("mix", {water: 10}) -> true
addRecipe("mix", {coffee: 4}) -> true        (replaces the earlier recipe)
dispense("mix") -> "OK"                      water 100, coffee 96
addRecipe("nothing", {})          -> false   (no ingredients)
addRecipe("bad", {water: 0})      -> false   (quantity must be positive)
addRecipe("worse", {water: -3})   -> false
dispense("bad") -> "UNKNOWN_DRINK"           (rejected recipes are not stored)
```

**4. Low stock** (water 100, milk 3, coffee 4, sugar 10)

| Call | Result |
|---|---|
| `lowStock(5)` | `["coffee", "milk"]` |
| `lowStock(4)` | `["milk"]` (4 is not strictly below 4) |
| `lowStock(1)` | `[]` |

**5. Several outlets racing** (milk 50, recipe "latte" = 5 milk, 40 simultaneous `dispense("latte")`): exactly 10 return `OK`, exactly 30 return `INSUFFICIENT:milk`, and final milk stock is 0.

## Edge cases

- Two drinks that share an ingredient compete for it: with water 50 and two recipes of 30 water each, the first dispense is `OK` and the second is `INSUFFICIENT:water`.
- Several ingredients short at once: the reported name is the alphabetically first short one (e.g. short on both `milk` and `coffee` gives `INSUFFICIENT:coffee`).
- Consuming an ingredient down to 0 keeps it known, so `lowStock` still lists it (0 is below any positive threshold).
- Under concurrent mixed drinks, final stock always equals the starting stock minus what the successful dispenses consumed, and never goes negative.

## Glossary

- **Ingredient**: a named consumable with an integer unit count.
- **Recipe**: a drink name mapped to the units of each ingredient it needs.
- **Outlet**: one place a user can request a drink from; several may be used at the same time.
- **Low stock**: an ingredient whose current units are strictly less than the threshold you pass.

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `dispense` returns exactly `OK`, `UNKNOWN_DRINK`, or `INSUFFICIENT:<ingredient>` (first missing ingredient in alphabetical order; an ingredient never refilled has stock 0).
- `addRecipe` returns `false` (and changes nothing) for an empty ingredient set or any quantity ≤ 0; `true` otherwise.
- `stock(ingredient)` is 0 for unknown ingredients.
- `lowStock(threshold)` lists ingredients whose stock is **strictly below** the threshold, sorted alphabetically. It includes ingredients that were refilled and then fully consumed.
- All operations must be safe to call concurrently from multiple threads.
- The drink name in `dispense` is matched exactly (case-sensitive) against recipe names.
- When several ingredients are short, the single name returned in `INSUFFICIENT:<ingredient>` is the alphabetically smallest short ingredient.
- Replacing a recipe affects later dispenses only; stock already consumed is not refunded.

## Test entry point (Java)

The tests drive your system through the thin, primitive-typed contract below. It is **not** your domain
model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in
`solution/Solution.java`.

```java
package contract;

import java.util.List;
import java.util.Map;

/** Thin test entry point. Your domain model lives behind it. Must be thread-safe. */
public interface CoffeeMachineApi {

    void refill(String ingredient, int units);

    /** Defines or replaces a recipe. Returns false if the recipe is invalid. */
    boolean addRecipe(String drink, Map<String, Integer> ingredients);

    /** "OK", "UNKNOWN_DRINK" or "INSUFFICIENT:&lt;ingredient&gt;". */
    String dispense(String drink);

    int stock(String ingredient);

    /** Ingredients with stock strictly below the threshold, sorted alphabetically. */
    List<String> lowStock(int threshold);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    CoffeeMachineApi create()
}
```

## Language notes

Contract methods are called from many threads in the tests. A recipe's ingredient map is passed in by the caller — decide whether you may keep a reference to it.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/04-coffee-machine test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

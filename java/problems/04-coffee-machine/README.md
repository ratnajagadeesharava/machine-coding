# 04 · Coffee Machine (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 60 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Model a configurable coffee machine that prepares drinks from a pool of ingredients. The recipes and the stock are
not hard-coded: operators define recipes and refill ingredients at runtime.

## Requirements

1. Ingredients are named (`water`, `milk`, `coffee`, …) and tracked as integer unit counts.
2. A **recipe** is a drink name plus the units of each ingredient it needs. Operators can add recipes at runtime; adding a recipe with an existing name replaces it.
3. Operators **refill** ingredients; refills accumulate.
4. A user can **dispense** a drink. It succeeds only if *every* ingredient is sufficient. A successful dispense consumes the ingredients; a failed one consumes **nothing**.
5. When a dispense fails because of stock, the result names the missing ingredient (the alphabetically first one if several are short).
6. Operators can list the ingredients that are **running low** (stock strictly below a threshold).
7. The machine has several **outlets** that may serve users at the same time. Stock must never go negative, and no ingredient may be consumed for a drink that was refused.

## Constraints

- Any number of recipes and ingredients; recipe size is small (< 20 ingredients).
- Concurrent dispenses are normal: dispensing different drinks should not needlessly serialize unrelated work, but correctness is non-negotiable.
- A recipe must have at least one ingredient and every quantity must be positive, otherwise it is rejected.

## Examples

```
refill(water, 100); refill(coffee, 10); refill(milk, 5)
addRecipe("latte", {water: 30, coffee: 2, milk: 5}) → true
dispense("latte") → "OK"       stock(milk) = 0
dispense("latte") → "INSUFFICIENT:milk"      (nothing consumed)
dispense("tea")   → "UNKNOWN_DRINK"
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `dispense` returns exactly `OK`, `UNKNOWN_DRINK`, or `INSUFFICIENT:<ingredient>` (first missing ingredient in alphabetical order; an ingredient never refilled has stock 0).
- `addRecipe` returns `false` (and changes nothing) for an empty ingredient set or any quantity ≤ 0; `true` otherwise.
- `stock(ingredient)` is 0 for unknown ingredients.
- `lowStock(threshold)` lists ingredients whose stock is **strictly below** the threshold, sorted alphabetically. It includes ingredients that were refilled and then fully consumed.
- All operations must be safe to call concurrently from multiple threads.

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

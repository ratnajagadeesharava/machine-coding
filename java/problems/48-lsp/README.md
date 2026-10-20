# 48 · LSP — Liskov Substitution Principle (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — the `Coupon` contract is provided; the rest is yours to write. |

## Problem

**What you are building.** A web shop issues several kinds of coupons: a percentage off, a fixed amount off, and "spend at least X to get Y off". The checkout adds up a basket and applies whichever coupon the customer has, and it must work with any coupon without knowing which kind it is holding. That only works if every coupon honours the same promises (never raises the price, never goes below zero, never fails just because the coupon does not apply). You implement the coupon kinds so they all keep those promises, plus the checkout.

## Requirements

Provided: interface `Coupon` in `contract/Coupon.java` with `String code()` and `long apply(long amountCents)` (price after discount). You write `PercentCoupon`, `FlatCoupon`, `MinSpendCoupon` and `Checkout`. Amounts are whole cents.

The contract every coupon must obey, in plain terms:

- `code()` is a non-blank string (it returns the code given at construction).
- For any `amount >= 0`, `apply(amount)` returns the price after the discount, with `0 <= result <= amount`. It **never throws** for such an amount, even if the coupon "does not apply" to it.
- `apply(0)` is 0.
- A negative amount throws `IllegalArgumentException`, exactly that type (not a subclass, not another exception), from every coupon.
- Deterministic: the same amount always gives the same result.

Behaviour of each coupon:

1. `new PercentCoupon(code, percent)`: `percent` must be in `1..100`, otherwise `IllegalArgumentException`. `apply(amount)` = `amount − discount`, where `discount = amount × percent / 100` rounded **down** (so the customer pays the rounded-up remainder).
2. `new FlatCoupon(code, offCents)`: `offCents >= 0` (negative → `IllegalArgumentException`). `apply(amount) = max(0, amount − offCents)`. A huge discount (even `Long.MAX_VALUE`) just gives 0.
3. `new MinSpendCoupon(code, minSpendCents, offCents)`: if `amount >= minSpendCents` → `max(0, amount − offCents)`; if `amount` is below the minimum → the amount unchanged. Below the minimum is **not** an error: nothing is thrown.
4. All coupons: a negative amount → `IllegalArgumentException` (exactly that type).
5. `Checkout.total(List<Long> pricesCents, Coupon coupon)` returns `long`: sum the prices, then apply the coupon **once** to that sum. A `null` coupon means no discount. A `null` or empty list gives 0 (whatever the coupon). Any negative price → `IllegalArgumentException`. `Checkout` works with any `Coupon`, including ones it has never seen (the tests define a `Stacked` coupon that applies one coupon and then another).

## How it is tested

`CouponContractTest` is one shared suite. It is run against `PercentCoupon`, `FlatCoupon`, `MinSpendCoupon` **and
a `Stacked` coupon defined in the test** (`StackedCouponTest`, built from your `PercentCoupon` and `FlatCoupon`). The same six contract tests must pass for each.

## Examples

Coupons:

```
new PercentCoupon("A", 25).apply(1001)  -> 751      // discount = 1001*25/100 = 250.25 -> 250
new PercentCoupon("C", 100).apply(777)  -> 0
new FlatCoupon("A", 500).apply(1000)    -> 500
new FlatCoupon("A", 500).apply(120)     -> 0        // clamped, never negative
m = new MinSpendCoupon("BIG", 1000, 300)
m.apply(999) -> 999      m.apply(1000) -> 700      m.apply(5000) -> 4700      m.apply(0) -> 0
new PercentCoupon("X", 0) / ("X", 101) / new FlatCoupon("X", -1)  -> IllegalArgumentException
anyCoupon.apply(-1) -> IllegalArgumentException
```

Checkout:

```
Checkout.total(List.of(600L, 600L), new FlatCoupon("F", 500))    -> 700     // (600+600) - 500
Checkout.total(List.of(500L, 500L), new PercentCoupon("P", 10)) -> 900     // 1000 - 100
Checkout.total(List.of(300L, 200L), null)                        -> 500     // no coupon
Checkout.total(List.of(300L, 200L), PercentCoupon 50%)           -> 250
Checkout.total(List.of(300L, 200L), FlatCoupon 100)              -> 400
Checkout.total(List.of(300L, 200L), MinSpendCoupon(5000, 100))   -> 500     // 500 < 5000, unchanged
Checkout.total(List.of(), anyCoupon) / Checkout.total(null, anyCoupon) -> 0
Checkout.total(Arrays.asList(100L, -1L), null)                   -> IllegalArgumentException
Checkout.total(List.of(600L, 400L), Stacked(10% then 100 off))   -> 800     // 1000 -> 900 -> 800
```

## Edge cases

- A discount can never push a price below 0: it is clamped at 0.
- A coupon that does not apply (below the minimum spend) is a normal case returning the unchanged amount, not an error.
- The coupon is applied to the **sum**, not to each price: two prices of 600 with a 500 flat coupon give 700, not 200.
- The discount is rounded down, so the shopper never gets more than the exact percentage off.

## Glossary

- **Coupon**: a discount rule with a code that turns a price into a lower or equal price.
- **Minimum spend**: the smallest basket amount at which a minimum-spend coupon starts to apply.
- **Substitutable**: any coupon, whatever its kind, can be used wherever a `Coupon` is expected without surprising the caller.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

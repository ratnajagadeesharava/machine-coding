# 48 · LSP — Liskov Substitution Principle (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — the `Coupon` trait, its documented contract and `CouponError` are provided; the rest is yours to write. |

## Problem

**What you are building.** A web shop issues several kinds of coupons: a percentage off, a fixed amount off, and "spend at least X to get Y off". The checkout adds up a basket and applies whichever coupon the customer has, and it must work with any coupon without knowing which kind it is holding. That only works if every coupon honours the same promises (never raises the price, never goes below zero, never fails just because the coupon does not apply). You implement the coupon kinds so they all keep those promises, plus the checkout.

## Requirements

Provided in `src/contract.rs`: trait `Coupon` with `fn code(&self) -> &str` and `fn apply(&self, amount_cents: i64) -> Result<i64, CouponError>` (price after discount), and `CouponError { NegativeAmount, InvalidArgument }`. You write `PercentCoupon`, `FlatCoupon`, `MinSpendCoupon` and the free function `checkout_total`. Amounts are whole cents.

The contract every coupon must obey, in plain terms (also at the top of `src/contract.rs`):

- `code()` is a non-blank string (it returns the code given at construction).
- For any `amount >= 0`, `apply(amount)` returns `Ok(price)` with `0 <= price <= amount`. It **never returns `Err`** for such an amount, even if the coupon "does not apply" to it.
- `apply(0)` is `Ok(0)`.
- A negative amount gives `Err(CouponError::NegativeAmount)` from every coupon.
- Deterministic: the same amount always gives the same result.

Behaviour of each coupon:

1. `PercentCoupon::new(code, percent)`: `percent` must be in `1..=100`, otherwise `Err(CouponError::InvalidArgument)`. `apply(amount)` = `amount − discount`, where `discount = amount × percent / 100` rounded **down** (so the customer pays the rounded-up remainder).
2. `FlatCoupon::new(code, off_cents)`: `off_cents >= 0` (negative → `Err(CouponError::InvalidArgument)`). `apply(amount) = max(0, amount − off_cents)`. A huge discount (even `i64::MAX`) just gives 0.
3. `MinSpendCoupon::new(code, min_spend_cents, off_cents)`: if `amount >= min_spend_cents` → `max(0, amount − off_cents)`; if `amount` is below the minimum → the amount unchanged. Below the minimum is **not** an error: it is `Ok(amount)`.
4. All coupons: a negative amount → `Err(CouponError::NegativeAmount)`.
5. `checkout_total(prices_cents: &[i64], coupon: Option<&dyn Coupon>) -> Result<i64, CouponError>`: sum the prices, then apply the coupon **once** to that sum. `None` means no discount. An empty slice gives `Ok(0)`. Any negative price → `Err(CouponError::NegativeAmount)`. It works with any `Coupon`, including ones it has never seen (the tests define a `Stacked` coupon that applies one coupon and then another).

## How it is tested

`contract_suite(&dyn Coupon)` in the test file is one shared set of checks. It is run against `PercentCoupon`,
`FlatCoupon`, `MinSpendCoupon` **and a `Stacked` coupon defined in the test** (built from your `PercentCoupon` and `FlatCoupon`).

## Examples

Coupons:

```
PercentCoupon::new("A", 25)?.apply(1001)  -> Ok(751)    // discount = 1001*25/100 = 250.25 -> 250
PercentCoupon::new("C", 100)?.apply(777)  -> Ok(0)
FlatCoupon::new("A", 500)?.apply(1000)    -> Ok(500)
FlatCoupon::new("A", 500)?.apply(120)     -> Ok(0)     // clamped, never negative
m = MinSpendCoupon::new("BIG", 1000, 300)?
m.apply(999) -> Ok(999)   m.apply(1000) -> Ok(700)   m.apply(5000) -> Ok(4700)   m.apply(0) -> Ok(0)
PercentCoupon::new("X", 0) / ("X", 101) / FlatCoupon::new("X", -1)  -> Err(InvalidArgument)
any_coupon.apply(-1) -> Err(NegativeAmount)
```

Checkout:

```
checkout_total(&[600, 600], Some(&FlatCoupon 500))   -> Ok(700)    // (600+600) - 500
checkout_total(&[500, 500], Some(&PercentCoupon 10))  -> Ok(900)    // 1000 - 100
checkout_total(&[300, 200], None)                      -> Ok(500)    // no coupon
checkout_total(&[300, 200], Some(&PercentCoupon 50))   -> Ok(250)
checkout_total(&[300, 200], Some(&FlatCoupon 100))     -> Ok(400)
checkout_total(&[300, 200], Some(&MinSpendCoupon(5000, 100))) -> Ok(500)  // 500 < 5000, unchanged
checkout_total(&[], Some(&any_coupon))                 -> Ok(0)
checkout_total(&[100, -1], None)                       -> Err(NegativeAmount)
checkout_total(&[600, 400], Some(&Stacked(10% then 100 off))) -> Ok(800)  // 1000 -> 900 -> 800
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
cargo test -p p48_lsp     # from rust/ — tests FAIL until you implement the stubs
cargo play run 48
```

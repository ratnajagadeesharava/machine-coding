# 139 · Shopping Cart (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 60 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

An online shop sells products from a catalogue. Shoppers fill a **cart**, the shop works out what it costs (item promotions, a coupon,
tax) and, at **checkout**, takes the goods out of stock and records an order. You are building the engine behind that: the catalogue
with stock, the carts, the exact price breakdown, the coupon rules and an atomic checkout. Prices are whole cents and every rounding
rule is spelled out below, so two correct solutions print the same numbers. Many shoppers use the shop at once; the last unit of a
product must never be sold twice.

## Requirements

All money is whole cents (a 64-bit integer). All divisions below are integer divisions that round down; no amount is ever negative.

### Catalogue

1. `add_product(sku, name, priceCents, category, stock)` registers a product. `priceCents > 0`, `stock >= 0`; sku, name and category must not be blank; a second product with an existing sku is an error. A sku is case-sensitive.
2. `set_stock(sku, n)` replaces the units on hand (`n >= 0`); `stock(sku)` reads them. Unknown sku is an error.
3. `set_tax_rate(category, basisPoints)` gives a category a tax rate of `0..=10000` basis points (100 bp = 1%). A category without a rate is taxed at 0.
4. A product has **at most one promotion**; setting a promotion replaces the previous one and `clear_promotion(sku)` removes it (no-op if there is none). Two kinds:
   - `set_buy_x_get_y(sku, x, y)` (`x >= 1`, `y >= 1`): in every complete group of `x + y` units of the sku in a cart, `y` units are free. The remainder (fewer than `x + y` units) pays full price. The free units cost `price` each, so the discount is `(qty / (x + y)) * y * price`.
   - `set_percent_off(sku, percent)` (`percent` from 1 to 90): the discount is `lineSubtotal * percent / 100`, rounded down.

   Promotions are looked up when a total is computed, so changing a promotion changes the totals of carts that already hold the product.

### Coupons

5. `define_coupon(code, kind, value, minSubtotalCents, maxUses)` defines a coupon. `kind` is `PERCENT` (`value` 1 to 100) or `FIXED` (`value >= 1` cents); `minSubtotalCents >= 0`; `maxUses >= 1`; the code is not blank and not yet defined (case-sensitive). `coupon_uses(code)` returns how many checkouts have consumed it so far (starts at 0).
6. A cart holds **at most one** coupon. `apply_coupon(cartId, code)` attaches a defined coupon and replaces the cart's previous one; it does not check eligibility. `remove_coupon(cartId)` detaches it (no-op if none).
7. A cart's coupon is **eligible** at a given moment when both hold: the cart's *post-promotion subtotal* (below) is at least `minSubtotalCents`, and the coupon has been consumed fewer than `maxUses` times. `coupon_status(cartId)` returns `NONE` (no coupon attached), `APPLIED` (attached and eligible) or `NOT_ELIGIBLE` (attached, not eligible). A coupon that is not eligible is ignored by `total` (discount 0) and by `checkout`, which then neither fails nor consumes a use.

### Carts

8. `create_cart(cartId, userId)`: both not blank; a second cart with the same id is an error.
9. `add_item(cartId, sku, qty)` (`qty >= 1`) adds units to the cart's line for the sku (creating the line when there is none). If the line's resulting quantity would be greater than the units on hand, the call is an error whose message contains `insufficient stock` and nothing changes. Stock is **not reserved** by carts: any number of carts can hold the same last unit.
10. `set_quantity(cartId, sku, qty)` (`qty >= 0`) sets the line to exactly `qty`; `0` removes the line (and is a no-op when there is no line). A quantity above the units on hand is the same `insufficient stock` error.
11. `remove_item(cartId, sku)` removes the line; it is an error when the cart has no line for the sku. `quantity(cartId, sku)` returns the units of the sku in the cart (0 when there is no line).
12. Unknown cart, sku or coupon ids are errors; a failing call changes nothing.

### Pricing — `total(cartId)`

`total` returns one string `subtotal|promoDiscount|couponDiscount|tax|total`, all in cents, computed in this exact order:

1. **Line subtotal** = `price * qty` for each line. `subtotal` is the sum of the line subtotals.
2. **Item promotion** per line (requirement 4). The line's amount after promotion is `L = lineSubtotal - promoDiscount_line`. `promoDiscount` is the sum of the line discounts, and the **post-promotion subtotal** is `S = subtotal - promoDiscount`.
3. **Coupon** (only if eligible, see 7), on `S`: `PERCENT` takes `S * value / 100`; `FIXED` takes `min(value, S)`. That is `couponDiscount` (0 when there is no eligible coupon).
4. **Tax.** The coupon is spread over the lines in proportion to their post-promotion amounts, then each line is taxed:
   - For every line `i` (in ascending sku order) compute `share_i = couponDiscount * L_i / S` rounded down and its remainder `r_i = couponDiscount * L_i mod S`.
   - The cents still unassigned, `couponDiscount - sum(share_i)`, are handed out one cent at a time to the lines with the **largest remainder** `r_i`; ties go to the line with the lowest sku (ascending string order). A line receives at most one extra cent.
   - Line tax = `(L_i - share_i) * basisPoints / 10000` rounded down, where `basisPoints` is the rate of the product's category. `tax` is the sum of the line taxes.
5. `total = subtotal - promoDiscount - couponDiscount + tax`.

An empty cart totals `0|0|0|0|0` (its coupon status is `NONE` or, with a coupon attached, follows requirement 7 with `S = 0`).

### Checkout

13. `checkout(cartId)` is **atomic**:
    - an empty cart is an error;
    - every line's quantity is compared with the units on hand; if any line lacks stock the call fails with the error `insufficient stock: <sku>` naming the **first failing sku in ascending sku order**, and *nothing* changes (stock, cart, coupon, orders);
    - otherwise stock is decremented for every line, the cart's coupon use is consumed **if the coupon is eligible at that moment**, the order is recorded with the total computed as `total` would have printed it just before, and the cart is emptied (lines **and** coupon). It returns the new order id.
14. Order ids are `ORD-1`, `ORD-2`, ... in the order checkouts succeed (one sequence for the whole shop, starting at 1).
15. `orders(userId)` lists the user's orders in checkout order, each as `orderId|totalCents` (for example `ORD-3|5800`); a user without orders gets an empty list; a blank user id is an error.
16. The service is used by many threads at once. Two carts that check out the last unit at the same time: exactly one succeeds. A coupon with `maxUses = 3` is consumed by exactly three of any number of simultaneous checkouts; the others are priced without it.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Every method returns `Result<_, CartError>`; a call that returns an error changes nothing. Which error:
  - `InvalidArgument`: a blank string, or a number outside the stated range (price, value, minimum, percent, x, y, basis points, quantity 0 for `add_item`, max uses 0), a blank user id for `orders`.
  - `UnknownSku`, `UnknownCart`, `UnknownCoupon`: the id does not exist. When both a cart and a sku are unknown, the tests do not say which is reported.
  - `AlreadyExists`: a duplicate sku, cart id or coupon code.
  - `NoSuchLine`: `remove_item` of a sku the cart has no line for.
  - `InsufficientStock(sku)`: `add_item`/`set_quantity` beyond the units on hand, and `checkout` with the first lacking sku in ascending order.
  - `EmptyCart`: `checkout` of a cart without lines.
- Money is `i64` cents; counts are `u32`. `total` returns `subtotal|promoDiscount|couponDiscount|tax|total`; `orders` returns `orderId|totalCents` strings. `kind` is `CouponKind::{Percent, Fixed}`; `coupon_status` returns `CouponStatus::{None, Applied, NotEligible}`.
- The service is used through `&self` from many threads (`Send + Sync`).

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract in `src/contract.rs`. It is **not** your domain model — it is the doorway. Add your own modules and types and return them from `create()` in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`). All money is in whole cents.
//! A call that returns an error changes nothing.

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum CartError {
    /// blank string, or a number outside the documented range
    InvalidArgument,
    /// a sku, cart id or coupon code that does not exist
    UnknownSku,
    UnknownCart,
    UnknownCoupon,
    /// duplicate sku, cart id or coupon code
    AlreadyExists,
    /// `remove_item` on a sku the cart has no line for
    NoSuchLine,
    /// not enough units on hand; carries the offending sku
    InsufficientStock(String),
    /// checkout of a cart without lines
    EmptyCart,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CouponKind {
    /// value 1..=100
    Percent,
    /// value >= 1 cent
    Fixed,
}

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CouponStatus {
    None,
    Applied,
    NotEligible,
}

pub trait CartService: Send + Sync {
    // ---- catalogue ----

    /// `price_cents > 0`; non-blank sku/name/category; a second product with the same sku is `AlreadyExists`.
    fn add_product(
        &self,
        sku: &str,
        name: &str,
        price_cents: i64,
        category: &str,
        stock: u32,
    ) -> Result<(), CartError>;

    /// Replaces the units on hand.
    fn set_stock(&self, sku: &str, n: u32) -> Result<(), CartError>;

    /// Units currently on hand.
    fn stock(&self, sku: &str) -> Result<u32, CartError>;

    /// Tax rate of a category in basis points, 0..=10000. Categories without a rate are taxed 0.
    fn set_tax_rate(&self, category: &str, basis_points: u32) -> Result<(), CartError>;

    /// In every complete group of x+y units, y units are free. x >= 1, y >= 1. Replaces any promotion of the sku.
    fn set_buy_x_get_y(&self, sku: &str, x: u32, y: u32) -> Result<(), CartError>;

    /// Integer percent 1..=90 off the line subtotal. Replaces any promotion of the sku.
    fn set_percent_off(&self, sku: &str, percent: u32) -> Result<(), CartError>;

    /// Removes the promotion of the sku (no-op if there is none).
    fn clear_promotion(&self, sku: &str) -> Result<(), CartError>;

    // ---- coupons ----

    /// `min_subtotal_cents >= 0`, `max_uses >= 1`; a second coupon with the same code is `AlreadyExists`.
    fn define_coupon(
        &self,
        code: &str,
        kind: CouponKind,
        value: i64,
        min_subtotal_cents: i64,
        max_uses: u32,
    ) -> Result<(), CartError>;

    /// Successful checkouts that have consumed the coupon so far.
    fn coupon_uses(&self, code: &str) -> Result<u32, CartError>;

    // ---- carts ----

    /// A second cart with the same id is `AlreadyExists`.
    fn create_cart(&self, cart_id: &str, user_id: &str) -> Result<(), CartError>;

    /// Adds qty (>= 1) units to the sku's line. Beyond the units on hand: `InsufficientStock(sku)`, cart unchanged.
    fn add_item(&self, cart_id: &str, sku: &str, qty: u32) -> Result<(), CartError>;

    /// Sets the line to exactly qty units; 0 removes the line. Same stock rule as `add_item`.
    fn set_quantity(&self, cart_id: &str, sku: &str, qty: u32) -> Result<(), CartError>;

    /// Removes the sku's line; `NoSuchLine` if the cart has none.
    fn remove_item(&self, cart_id: &str, sku: &str) -> Result<(), CartError>;

    /// Units of the sku in the cart (0 if no line).
    fn quantity(&self, cart_id: &str, sku: &str) -> Result<u32, CartError>;

    /// Replaces the cart's coupon (the code must be defined). Eligibility is not checked here.
    fn apply_coupon(&self, cart_id: &str, code: &str) -> Result<(), CartError>;

    /// Detaches the cart's coupon (no-op if none).
    fn remove_coupon(&self, cart_id: &str) -> Result<(), CartError>;

    /// Evaluated now.
    fn coupon_status(&self, cart_id: &str) -> Result<CouponStatus, CartError>;

    /// "subtotal|promoDiscount|couponDiscount|tax|total" in cents, evaluated now.
    fn total(&self, cart_id: &str) -> Result<String, CartError>;

    /// Atomically: check stock of every line, decrement stock, consume the coupon use (if eligible), empty the cart
    /// (coupon included), record the order. Returns "ORD-1", "ORD-2", ... in checkout order.
    /// `InsufficientStock(sku)` names the first lacking sku in ascending sku order; `EmptyCart` for a cart without lines.
    fn checkout(&self, cart_id: &str) -> Result<String, CartError>;

    /// The user's orders in checkout order, each "orderId|totalCents". Empty for a user without orders.
    fn orders(&self, user_id: &str) -> Result<Vec<String>, CartError>;
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any design behind it works. No clock is involved. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Catalogue and cart lines: stock, invalid input, merging lines, `InsufficientStock` and an unchanged cart, no reservation, `set_quantity`, `remove_item`, unknown ids.
- Pricing: both promotion kinds and their rounding, one promotion per sku, percent and fixed coupons, replacing a coupon, minimum subtotal after promotions, per-line tax, coupon spread over lines, the README worked example.
- Checkout: stock, cart and order effects, the first failing sku in sku order, empty cart, coupon consumption and exhaustion, per-user order lists.
- Concurrency: the last unit sold once, no overselling, a coupon's `max_uses` under simultaneous checkouts.
- More (large amounts, opposite lock orders, atomicity under contention) in `interviewer/`.

## Examples

### Prices (worked end to end)

```
add_product("A", "Anvil", 1000, "goods", 20)      add_product("B", "Bolt", 333, "food", 20)
set_tax_rate("goods", 1000)                         set_tax_rate("food", 500)
set_buy_x_get_y("A", 2, 1)                            set_percent_off("B", 10)
define_coupon("SAVE10", PERCENT, 10, 5000, 3)
create_cart("c1", "u1"); add_item("c1","A",7); add_item("c1","B",3); apply_coupon("c1","SAVE10")

line A: 7 * 1000 = 7000; groups of 3: 7 / 3 = 2 -> 2 free units = 2000 off -> 5000
line B: 3 * 333  = 999;  10% -> 999 * 10 / 100 = 99 off                   -> 900
subtotal 7999, promoDiscount 2099, S = 5900
coupon: eligible (5900 >= 5000, 0 of 3 uses) -> 5900 * 10 / 100 = 590
shares: A 590 * 5000 / 5900 = 500, B 590 * 900 / 5900 = 90 (no remainders)
tax: A (5000-500) * 1000 / 10000 = 450;  B (900-90) * 500 / 10000 = 40 (40.5 rounds down)
total("c1")        -> "7999|2099|590|490|5800"
coupon_status("c1") -> APPLIED
```

### Spreading the coupon over lines

```
three products a, b, c at 100 cents, category "x" taxed at 1000 bp; one of each in the cart
define_coupon("OFF100", FIXED, 100, 0, 5); apply_coupon(...)

S = 300, couponDiscount = min(100, 300) = 100
shares: 100 * 100 / 300 = 33 each, remainder 100 each -> 99 assigned, 1 cent left
all remainders tie -> lowest sku "a" gets it: shares 34, 33, 33
taxable 66, 67, 67 -> tax 6 + 6 + 6 = 18
total -> "300|0|100|18|218"        (taxing the whole 200 at once would give 20: the rule is per line)
```

### The last unit

```
add_product("ps5", "Console", 49900, "tech", 1)
cart c1 (user u1) and cart c2 (user u2) each add_item(.., "ps5", 1)     -> both succeed, stock("ps5") is still 1
checkout("c1")  -> "ORD-1"        stock("ps5") -> 0       orders("u1") -> ["ORD-1|49900"]
checkout("c2")  -> error "insufficient stock: ps5"        quantity("c2","ps5") -> 1 (the cart is untouched)
```

### A coupon that runs out

```
add_product("pen", "Blue pen", 1000, "stationery", 10); define_coupon("ONCE", PERCENT, 10, 0, 1)
carts c1 (u1) and c2 (u2) hold one pen each and both apply "ONCE"; coupon_status is APPLIED for both
checkout("c1")  -> "ORD-1"   orders("u1") -> ["ORD-1|900"]   coupon_uses("ONCE") -> 1   coupon_status("c1") -> NONE
coupon_status("c2") -> NOT_ELIGIBLE      total("c2") -> "1000|0|0|0|1000"
checkout("c2")  -> "ORD-2"   orders("u2") -> ["ORD-2|1000"]   coupon_uses("ONCE") -> 1   (not consumed again)
```

## Edge cases

- The minimum subtotal is compared with the amount **after** item promotions, and is satisfied when the amount is **equal** to it.
- Eligibility is evaluated every time it is asked: lowering a quantity can turn an `APPLIED` coupon into `NOT_ELIGIBLE` and back. Attaching a coupon never fails because it is not eligible.
- A `FIXED` coupon larger than the post-promotion subtotal takes the whole subtotal (total 0, and then no tax). A `PERCENT` coupon of 100 does the same.
- A coupon whose discount rounds to 0 (a tiny subtotal) is still *eligible*, and a checkout still consumes a use.
- With no eligible coupon `couponDiscount` is 0 and tax is simply `L_i * basisPoints / 10000` per line.
- Tax is rounded down **per line**, never on the sum, and the cart is not taxed on goods the coupon paid for.
- `add_item` and `set_quantity` compare with the stock *now*. If stock later drops (`set_stock`), the cart keeps its lines and the problem surfaces at `checkout`.
- `checkout` of a cart whose coupon is no longer eligible succeeds without the coupon; only a lacking line or an empty cart fails it.
- A failed checkout leaves the cart, its coupon, the stock, the coupon uses and the order numbering untouched (the next success is still the next number).
- Order ids are global; a user's list contains only that user's orders, in order.
- Amounts stay below 10^9 cents per cart in every test, so `cents * cents` fits in a 64-bit integer.

## Glossary

- **SKU**: the unique id of a product.
- **Line**: a (sku, quantity) pair in a cart; a cart has at most one line per sku.
- **Promotion**: a per-product discount applied to its line (buy-X-get-Y-free or percent off).
- **Post-promotion subtotal**: the cart's subtotal minus all promotion discounts; coupon and minimum are measured on it.
- **Coupon**: a cart-level discount with a minimum subtotal and a maximum number of uses across all carts.
- **Basis point**: one hundredth of a percent (1000 bp = 10%).
- **Largest remainder**: the way to divide a whole number of cents proportionally without losing or inventing a cent (see Pricing, step 4).
- **Reservation**: holding stock for a cart. Not part of this problem: stock is only taken at checkout.

## Running

```bash
cargo test               # from this directory (or `cargo test -p p139_shopping_cart` from rust/)
STRICT=1 cargo test      # fail instead of skip while unimplemented
cargo test --features hidden   # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation print `SKIPPED` and return until `create` is wired up, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`sut!`, `run_concurrently`): `rust/support`.

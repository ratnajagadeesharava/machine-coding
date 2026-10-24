# Hints — 139 Shopping Cart (Rust)

Try the problem for at least 15 minutes before opening a hint, and open **one level at a time**. Each level is hidden
behind a collapsed block. No level contains a full solution.

## Level 1

<details>
<summary>A small nudge — click to reveal</summary>

Start with the price of one cart and ignore checkout, coupons and threads. Write down, on paper, the numbers a worked example produces after each step (line subtotals, after promotions, after the coupon, tax). Which of those numbers must be remembered on the cart, and which can always be recomputed from the cart's lines and the catalogue?

</details>

## Level 2

<details>
<summary>Which tool, which direction — click to reveal</summary>

Keep the cart small (lines and at most one coupon code) and compute the breakdown on demand from it, in the order the README gives, as a function with no side effects. Both `total` and `checkout` then call the same function, so they cannot disagree. Make each kind of discount (a promotion kind, a coupon kind) something that answers "how many cents do you take off this input?", so that a new kind does not touch the others. For the coupon-over-lines step, first take the floor of each share, then count how many cents are missing and give them out in the order the README states. For checkout, think about what must not be interleaved with any other checkout: the stock comparison, the stock decrement and the coupon counter.

</details>

## Level 3

<details>
<summary>The shape of the solution — click to reveal</summary>

Shape: `Shop { products (price, category, stock), promotions by sku, tax by category, coupons (rule, uses), carts (user, lines sorted by sku, coupon code), orders by user, order counter }` and `price(cart) -> Breakdown { subtotal, promo, coupon, tax, total, eligible }`.

`price` = for each line (sorted by sku): subtotal and promotion discount; sum S; coupon discount if eligible; shares by floor plus the leftover cents to the largest remainders (sort by remainder descending, then sku ascending); tax per line on `L_i - share_i`. `checkout` = under one lock: empty? → check every line against stock in sku order → price → decrement all → increment the coupon use if `eligible` → append the order → clear lines and coupon.

**Java:** one `synchronized` service (or a `ReentrantLock`) around plain `HashMap`/`TreeMap` is enough to pass; keep money in `long`. **Rust:** a single `Mutex<State>` with `HashMap`/`BTreeMap` fields; keep the price function a plain `fn(&State, &Cart) -> Breakdown` so it can be called while the lock is held, and use `i64` for cents.

</details>

If you are still stuck after Level 3, write down which invariant you cannot keep and ask for a review of your attempt instead of a solution.

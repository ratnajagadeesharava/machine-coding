# Interviewer notes — 139 Shopping Cart

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Coupon spread: the cent that does not divide goes to the **largest remainder**, not to the first or the last line (`3100|0|1000|208|2308`); on equal remainders the lowest sku wins.
- A percent coupon whose per-line shares do not add up (`2553|0|178|250|2625`): the spare cent changes the tax of one line.
- Promotion, coupon and tax on several lines together (`10994|4000|123|1374|8245`), including a 90% promotion whose discount floors.
- Cents times cents overflow a 32-bit integer (`999999000` subtotal times a rate of 10000 bp).
- Order ids stay unique and gapless (`ORD-1..ORD-40`) when 40 carts check out at once.
- Carts that hold the same two skus and add them in opposite orders, checking out 300 times each on 16 threads: no deadlock, stock and order counts exact.
- 8 threads adding one unit at a time to the same cart against a stock of 500: exactly 500 are accepted.
- 24 carts compete for 5 units of a second sku: the 19 failures name that sku, take nothing from the first sku, consume no coupon use and keep their carts.

## Likely interviewer questions
- Where is the price computed, and could the cart total be shown without writing to anything?
- Why is the coupon eligibility not stored on the cart? What would break if it were?
- Why is the minimum subtotal compared after promotions and not before?
- What exactly is atomic about `checkout`, and which lock(s) does it hold? Why is a single lock around the whole shop acceptable here, and what would you do for 10,000 checkouts per second?
- Two carts check out the last unit: how do you show exactly one wins? Which assertion in your own tests proves it?
- What changes in your design if reservation with expiry (V2) arrives?

## Invariants
- Stock is never negative and never decreases outside a successful checkout (or `setStock`).
- The printed total equals `subtotal - promoDiscount - couponDiscount + tax` and the shares of the coupon add up to the coupon discount.
- A failed operation changes nothing: cart, stock, coupon uses, order sequence.
- A coupon use is consumed at most `maxUses` times in total, and only by a checkout that actually applied it.
- Order ids are unique and increase by one per successful checkout; a user's list is in checkout order.
- A cart has at most one line per sku and at most one coupon; a product at most one promotion.

## Common mistakes
- `double` or `float` for money, or percent as `0.1`, so that 999 at 10% is not exactly 99.
- Rounding the tax on the cart sum instead of per line; or computing the line tax on the line before the coupon share.
- Applying the coupon before the promotions, or checking the minimum before them.
- Distributing the remainder to the first/last line, or giving all remaining cents to one line.
- Storing "coupon applied" on the cart at apply time and forgetting to re-check at totals and checkout.
- Decrementing stock line by line and failing half-way (partial checkout); or checking stock with one lock and decrementing with another (check-then-act race).
- Reporting a lacking sku in insertion or hash order instead of ascending sku order.
- Consuming a coupon use for a checkout that failed, or for a coupon that was not eligible.
- Locking per cart and per product in inconsistent orders (deadlock), or locking only the cart and letting two carts oversell.
- Reserving stock at `addItem` (the requirement says it is not reserved), so a second cart could not add the last unit.
- Letting `setQuantity(.., 0)` fail when the line does not exist, or `removeItem` succeed silently.

## Over-engineering traps
- A rule engine, a plugin registry and a `DiscountStrategyFactory` before two promotions and one coupon work.
- An event bus for order placement, a repository layer and DTOs for a thin in-memory service.
- An inventory service with reservations, expiry and a scheduler (that is V2).
- Money and percent value types with operator overloading before the arithmetic is right (a small `Money` wrapper is fine if it earns its place).

## Patterns and structures an interviewer hopes to hear (only after the candidate has a working design)
- **Strategy** for the discount kinds: an item promotion (buy-X-get-Y, percent off) and a coupon (percent, fixed) are each "given a context, return a discount"; adding a new kind is a new strategy, not a new `if`.
- **Chain / pipeline of pricing steps** (line subtotal → promotions → coupon → tax), each step reading the previous step's result; a pricing result object that is the single source of `total` and `checkout`.
- A **value object for the price breakdown** that is computed on demand, never stored on the cart.
- **One lock for checkout** (or striped locks acquired in sorted sku order) to make the verify-and-decrement indivisible; a `Map` of products and a sorted map for lines make "first failing sku in sku order" free.
- Immutable snapshots of a cart for pricing so a total can be computed without holding the shop lock for long.

## Design and scaling discussion
- A single lock around catalogue, carts and orders is correct and simple; the first step towards scale is to price outside the lock from a snapshot and hold a lock only for the stock check and decrement.
- Per-sku locking, acquired in ascending sku order (the same order the error message uses), avoids deadlock and lets carts of different products check out in parallel.
- Coupon uses are a shared counter across carts: a compare-and-increment under the same lock as stock, or an atomic with a retry, keeps `maxUses` exact.
- In a distributed shop, stock and coupon counters become database rows with conditional updates (`UPDATE ... WHERE stock >= n`), and checkout becomes a transaction; mention idempotency keys for retried checkouts.
- Money in integer cents, with the largest-remainder rule, is the standard way to make parts add up to the whole.

## Trade-off questions to ask when the candidate is done
- What would you change to price a cart without holding the shop-wide lock?
- If stock lives in another service, how do you keep checkout atomic without a distributed transaction?
- Why spread the coupon over lines at all? What does the business gain, and what would break (returns, V5) without it?
- How would you prove to a customer that the total they saw is the total they were charged?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.

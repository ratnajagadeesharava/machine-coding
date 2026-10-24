# Rubric — 139 Shopping Cart (Java)

| Area | Weight |
|---|---:|
| Functional correctness | 45% |
| Design quality (model, API, responsibilities) | 20% |
| Code quality | 10% |
| Testing | 10% |
| Concurrency (declared scope, no shared mutable surprises) | 10% |
| Performance | 5% |

Score each area 0–4 (0 = absent, 2 = acceptable, 4 = would hire on this alone), multiply by weight.

## Questions for every problem

**Design**
- Did the candidate identify the correct domain concepts?
- Are responsibilities well separated; does each piece of state have one clear owner?
- Are abstractions justified by a requirement? (Not: "did they use pattern X?")
- Is the API clean, hard to misuse, and free of leaked internals?
- Can the requirements evolve (see `FOLLOWUPS.md`) without a rewrite?
- Is the implementation unnecessarily complicated?

**Correctness & quality**
- Are invariants preserved across every operation, including failed ones (no half-applied changes)?
- Are edge cases (empty, boundary, repeated, invalid operations) handled deliberately?
- Can components be tested independently of the whole system?

**Concurrency & performance**
- Is concurrency safe (or explicitly and correctly declared out of scope)? Is the locking granularity reasoned about?
- Are performance characteristics reasonable for the stated constraints?

**Language-specific**
- Are classes/interfaces justified, or did you reach for an interface by reflex?
- Are collections, locks and executors chosen on purpose (not `synchronized` everywhere)?
- Is mutable state encapsulated; are value types immutable (records / final fields)?

## Problem-specific

- Is the price computed in the specified order (line subtotal, item promotion, coupon on the post-promotion subtotal, tax) with every rounding exactly as stated?
- Is the coupon spread over lines by largest remainder with the stated tie-break, so that the parts add up to the printed total?
- Is eligibility of the coupon evaluated at the time of the question (totals and checkout) rather than frozen when it was applied?
- Is checkout all-or-nothing: stock check for every line, stock decrement, coupon use and order recording in one indivisible step, with the first failing sku reported in sku order?
- Does the design avoid selling the last unit twice and avoid deadlock when carts touch the same skus in different orders?
- Is the price breakdown separated from the cart's storage so that a follow-up (shipping, stacking coupons) adds a step instead of rewriting one?

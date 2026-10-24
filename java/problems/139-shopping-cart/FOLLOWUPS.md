# Follow-ups — 139 Shopping Cart

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Reservation with expiry
Adding to a cart now holds the units for 15 minutes; a second shopper cannot take them, and the hold disappears when it expires or the cart is emptied. Where does the clock come from, who releases an expired hold (a sweeper, or lazily on the next read), and what does `checkout` do with a hold that expired a moment ago?

### V3 — Several coupons that stack
A cart may hold several coupons, some of which are *stackable* and some *exclusive*, with an order of application and a cap on the total discount. Which part of your pricing changes, which part stays, and how do you keep "a coupon's minimum is checked on the amount it sees" well-defined when two coupons apply one after the other?

### V4 — Shipping fees and tax on shipping
Add a shipping fee by weight band that is waived above a threshold (measured before or after discounts: pick one and defend it), and taxed at a rate of its own. How do you add a new price component without touching every existing one?

### V5 — Returns and refunds
A customer returns two of five units from an order. What is refunded when a coupon and a buy-X-get-Y promotion were part of that order, and what happens to the coupon use and to the stock? What must you have recorded at checkout to make this possible?

### V6 — Price changes while items are in carts
The price of a product changes while it sits in carts. Do carts re-price (and tell the shopper), or keep the price they were added at for a while? How does an order stay unchanged when prices change later?

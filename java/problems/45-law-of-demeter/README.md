# 45 · Law of Demeter (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — the `Wallet`, `Customer` and `Order` interfaces are provided; you write two implementations and two services. |

## Problem

**What you are building.** A small shop back end with two services. A *cashier* charges a customer for a purchase: if the customer can afford it, the money is taken and the sale counts as revenue; if not, the sale is declined and nothing happens. A *shipping calculator* quotes the delivery price of an order from the order's value and destination city. Customers pay out of a wallet, but the cashier and the shipping calculator are only allowed to talk to the customer or order they are handed, never to the objects hidden behind them. The tests enforce this: the `Customer` used by the cashier tests throws if `getWallet()` is called, and the `Order` used by the shipping tests throws if `getCustomer()` is called.

All money amounts are whole cents (`long`).

## Requirements

Provided in `contract/`: `Wallet` (`balance()`, `debit(amount)`), `Customer` (`name()`, `canAfford(amount)`, `pay(amount)`, `getWallet()`) and `Order` (`getCustomer()`, `shippingCity()`, `itemTotal()`). You write `SimpleWallet`, `StandardCustomer`, `Cashier` and `ShippingCalculator`.

1. `SimpleWallet(initial)`: starts with balance `initial`. `initial < 0` → `IllegalArgumentException`. `balance()` returns the current balance.
   `debit(amount)` subtracts `amount` from the balance. `amount <= 0` → `IllegalArgumentException`; `amount` greater than the balance → `IllegalStateException` and the balance is unchanged. Debiting exactly the balance is allowed (balance becomes 0).
2. `StandardCustomer(name, wallet)` implements `Customer`:
   - `name()` returns the name given.
   - `canAfford(amount)` is true when the wallet balance is `>= amount` (exactly equal counts as affordable).
   - `pay(amount)` debits that wallet. If the customer cannot afford it → `IllegalStateException` and nothing is debited.
   - `getWallet()` returns the wallet that was passed in.
3. `Cashier.checkout(customer, amount)` returns a `String`:
   - `customer == null` or `amount <= 0` → `IllegalArgumentException` (nothing is charged).
   - If `customer.canAfford(amount)` is true → calls `customer.pay(amount)` exactly once, adds `amount` to revenue, returns `"APPROVED"`.
   - Otherwise returns `"DECLINED"`, never calls `pay`, revenue is unchanged.
   - `revenue()` is the sum of all approved amounts (0 for a new cashier). It is the cashier's own total across all customers.
4. `ShippingCalculator.cost(order)` returns the shipping price in cents:
   - `order.itemTotal() >= 10000` → 0 (free shipping; exactly 10000 is free, 9999 is not).
   - Otherwise by `order.shippingCity()`, compared case-insensitively: `Paris` 1500, `Lyon` 1800, any other city 2500.
   - Only the order's own `shippingCity()` and `itemTotal()` are needed; `getCustomer()` must not be called.

## Examples

Wallets and customers:

```
w = new SimpleWallet(1000); ann = new StandardCustomer("Ann", w)
ann.canAfford(1000) -> true       ann.canAfford(1001) -> false
ann.pay(400)                       w.balance() -> 600
ann.pay(601)  -> IllegalStateException, w.balance() still 600
new SimpleWallet(-1) -> IllegalArgumentException
w.debit(0)           -> IllegalArgumentException
```

Cashier (a fresh `Cashier` each row group):

| Call | Result | Revenue after |
|---|---|---|
| `checkout(customerWith500, 300)` | `"APPROVED"` (customer now has 200, `pay` called once) | 300 |
| `checkout(customerWith100, 300)` | `"DECLINED"` (`pay` never called) | 300 |
| `checkout(bob /*wallet 500*/, 500)` | `"APPROVED"` (wallet 0) | 500 |
| `checkout(bob, 1)` | `"DECLINED"` | 500 |
| `checkout(null, 100)` | `IllegalArgumentException` | unchanged |
| `checkout(anyCustomer, 0)` | `IllegalArgumentException` | unchanged |

Shipping:

| City | itemTotal | Cost |
|---|---|---|
| `"Paris"` | 5000 | 1500 |
| `"LYON"` | 5000 | 1800 |
| `"Berlin"` | 5000 | 2500 |
| `"Paris"` | 10000 | 0 |
| `"Berlin"` | 9999 | 2500 |

## Edge cases

- A declined checkout is a normal result, not an error: it returns `"DECLINED"` and does not throw.
- Invalid arguments are rejected before anything is charged.
- Revenue only grows from approved checkouts.
- A failed `pay` or `debit` leaves the balance untouched.

## Glossary

- **Wallet**: holds a customer's money as a balance in cents.
- **Revenue**: total of the amounts the cashier has approved so far.
- **Free-shipping threshold**: item total of 10000 cents (100.00) or more.
- **Law of Demeter**: a caller should talk only to the object it was given, not to that object's internals (here: not `customer.getWallet()` or `order.getCustomer()`).

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

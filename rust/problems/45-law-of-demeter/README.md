# 45 · Law of Demeter (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — the `Wallet`, `Customer`, `Order` traits and `Outcome`/`PayError` are provided in `src/contract.rs`; you write two implementations and two services. |

## Problem

**What you are building.** A small shop back end with two services. A *cashier* charges a customer for a purchase: if the customer can afford it, the money is taken and the sale counts as revenue; if not, the sale is declined and nothing happens. A *shipping calculator* quotes the delivery price of an order from the order's value and destination city. Customers pay out of a wallet, but the cashier and the shipping calculator are only allowed to talk to the customer or order they are handed, never to the objects hidden behind them. The tests enforce this: the `Customer` used by the cashier tests panics if `wallet()` is called, and the `Order` used by the shipping tests panics if `customer()` is called.

All money amounts are whole cents (`u64`).

## Requirements

Provided in `src/contract.rs`: traits `Wallet` (`balance()`, `debit(amount)`), `Customer` (`name()`, `can_afford(amount)`, `pay(amount)`, `wallet()`), `Order` (`customer()`, `shipping_city()`, `item_total()`), plus `PayError { InvalidAmount, InsufficientFunds }` and `Outcome { Approved, Declined }`. You write `SimpleWallet`, `StandardCustomer`, `Cashier` and `ShippingCalculator`. Amounts are `u64`, so a negative amount cannot exist; zero is the invalid amount.

1. `SimpleWallet::new(initial)`: starts with balance `initial` (never fails). `balance()` returns the current balance.
   `debit(amount)` subtracts `amount` from the balance. `amount == 0` → `Err(PayError::InvalidAmount)`; `amount` greater than the balance → `Err(PayError::InsufficientFunds)` and the balance is unchanged. Debiting exactly the balance is allowed (balance becomes 0).
2. `StandardCustomer::new(name, wallet)` (wallet is a `Box<dyn Wallet>`) implements `Customer`:
   - `name()` returns the name given.
   - `can_afford(amount)` is true when the wallet balance is `>= amount` (exactly equal counts as affordable).
   - `pay(amount)` debits that wallet. If the customer cannot afford it → `Err(PayError::InsufficientFunds)` and nothing is debited.
   - `wallet()` returns the wallet that was passed in.
3. `Cashier::new()`; `checkout(&mut dyn Customer, amount)` returns `Result<Outcome, PayError>`:
   - `amount == 0` → `Err(PayError::InvalidAmount)` (nothing is charged, `pay` not called).
   - If `can_afford(amount)` is true → calls `pay(amount)` exactly once, adds `amount` to revenue, returns `Ok(Outcome::Approved)`.
   - Otherwise returns `Ok(Outcome::Declined)`, never calls `pay`, revenue is unchanged.
   - `revenue()` is the sum of all approved amounts (0 for a new cashier). It is the cashier's own total across all customers.
4. `ShippingCalculator::new()`; `cost(&dyn Order)` returns the shipping price in cents:
   - `order.item_total() >= 10000` → 0 (free shipping; exactly 10000 is free, 9999 is not).
   - Otherwise by `order.shipping_city()`, compared case-insensitively: `Paris` 1500, `Lyon` 1800, any other city 2500.
   - Only the order's own `shipping_city()` and `item_total()` are needed; `customer()` must not be called.

## Examples

Wallets and customers:

```
ann = StandardCustomer::new("Ann", Box::new(SimpleWallet::new(1000)))
ann.can_afford(1000) -> true       ann.can_afford(1001) -> false
ann.pay(400) -> Ok(())             ann.wallet().balance() -> 600
ann.pay(601) -> Err(InsufficientFunds), balance still 600
SimpleWallet::new(10).debit(0)  -> Err(InvalidAmount)
SimpleWallet::new(10).debit(11) -> Err(InsufficientFunds), balance still 10
```

Cashier (a fresh `Cashier::new()` per group):

| Call | Result | Revenue after |
|---|---|---|
| `checkout(&mut customer_with_500, 300)` | `Ok(Approved)` (customer now has 200, `pay` called once) | 300 |
| `checkout(&mut customer_with_100, 300)` | `Ok(Declined)` (`pay` never called) | 300 |
| `checkout(&mut bob /* wallet 500 */, 500)` | `Ok(Approved)` (wallet 0) | 500 |
| `checkout(&mut bob, 1)` | `Ok(Declined)` | 500 |
| `checkout(&mut any_customer, 0)` | `Err(InvalidAmount)` | unchanged |

Shipping:

| City | itemTotal | Cost |
|---|---|---|
| `"Paris"` | 5000 | 1500 |
| `"LYON"` | 5000 | 1800 |
| `"Berlin"` | 5000 | 2500 |
| `"Paris"` | 10000 | 0 |
| `"Berlin"` | 9999 | 2500 |

## Edge cases

- A declined checkout is a normal result, not an error: it returns `Ok(Outcome::Declined)`, not an `Err`.
- Invalid arguments are rejected before anything is charged.
- Revenue only grows from approved checkouts.
- A failed `pay` or `debit` leaves the balance untouched.

## Glossary

- **Wallet**: holds a customer's money as a balance in cents.
- **Revenue**: total of the amounts the cashier has approved so far.
- **Free-shipping threshold**: item total of 10000 cents (100.00) or more.
- **Law of Demeter**: a caller should talk only to the object it was given, not to that object's internals (here: not `customer.wallet()` or `order.customer()`).

## Running

```bash
cargo test -p p45_law_of_demeter     # from rust/ — tests FAIL until you implement the stubs
cargo play run 45
```

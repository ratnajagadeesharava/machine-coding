# 45 · Law of Demeter (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — the `Wallet`, `Customer`, `Order` traits and `Outcome`/`PayError` are provided in `src/contract.rs`; two implementations and two services are stubs. |

> **Principle:** "talk to your friends, not to strangers." A function should use only its own data, its
> parameters, and what it creates. `customer.wallet().debit(x)` and `order.customer().address().city()` tie the
> caller to the whole chain.

## Problem

A checkout takes a payment from a customer and a shipping calculator prices delivery from an order. The model
exposes everything (`Customer::wallet()`, `Order::customer()`), which invites train-wreck code. Write the services so
they never reach through.

## Requirements

1. `SimpleWallet::new(initial)`; `debit(0)` → `Err(PayError::InvalidAmount)`; more than the balance → `Err(PayError::InsufficientFunds)` with the balance unchanged.
2. `StandardCustomer::new(name, wallet)` implements `Customer`: `can_afford` / `pay` are answered **by the customer** by delegating to its own wallet (`pay` beyond the funds → `Err(InsufficientFunds)`, nothing debited). `wallet()` returns the wallet.
3. `Cashier::checkout(&mut dyn Customer, amount)`: if `can_afford(amount)` → `pay(amount)`, add to revenue, `Ok(Outcome::Approved)`; otherwise `Ok(Outcome::Declined)` without calling `pay`. `amount == 0` → `Err(PayError::InvalidAmount)`. `revenue()` is the sum of approved amounts.
4. `ShippingCalculator::cost(&dyn Order)`: free (0) when `item_total() >= 10000`; otherwise by `shipping_city()` (case-insensitive): Paris 1500, Lyon 1800, anything else 2500.

## Constraints (what the tests catch)

- The tests give `Cashier` a customer whose `wallet()` panics, and `ShippingCalculator` an order whose `customer()` panics. Calling either — directly or in a chain — fails the test.

## Running

```bash
cargo test -p p45_law_of_demeter     # from rust/ — tests FAIL until you implement the stubs
cargo play run 45
```

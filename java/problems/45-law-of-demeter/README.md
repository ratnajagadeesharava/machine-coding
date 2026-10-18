# 45 · Law of Demeter (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — the `Wallet`, `Customer` and `Order` interfaces are provided; two implementations and two services are stubs. |

> **Principle:** "talk to your friends, not to strangers." A method should call methods only on itself, its
> parameters, objects it creates, and its direct fields. `customer.getWallet().debit(x)` and
> `order.getCustomer().getAddress().getCity()` make the caller depend on the whole chain.

## Problem

A checkout takes a payment from a customer and a shipping calculator prices delivery from an order. The existing
model exposes everything (`getWallet()`, `getCustomer()`), which invites train-wreck code. Write the services so
they never reach through.

## Requirements

1. `SimpleWallet(initial)`: `initial >= 0` else `IllegalArgumentException`. `debit(amount)`: `amount <= 0` → `IllegalArgumentException`; more than the balance → `IllegalStateException` (balance unchanged).
2. `StandardCustomer(name, wallet)` implements `Customer`: `canAfford` and `pay` are answered **by the customer** by delegating to its own wallet (`pay` beyond the funds → `IllegalStateException`, nothing debited). `getWallet()` returns the wallet.
3. `Cashier.checkout(customer, amount)`: if `customer.canAfford(amount)` → call `customer.pay(amount)`, add to revenue, return `"APPROVED"`; otherwise return `"DECLINED"` without calling `pay`. Null customer or `amount <= 0` → `IllegalArgumentException`. `revenue()` is the sum of approved amounts.
4. `ShippingCalculator.cost(order)`: free (0) when `order.itemTotal() >= 10000`; otherwise by `order.shippingCity()` (case-insensitive): Paris 1500, Lyon 1800, anything else 2500.

## Constraints (what the tests catch)

- The tests hand `Cashier` a customer whose `getWallet()` throws `AssertionError`, and `ShippingCalculator` an order whose `getCustomer()` throws. Calling either — directly or in a chain — fails the test.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

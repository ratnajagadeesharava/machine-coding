# 45 · Law of Demeter (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — the `Wallet`, `Customer` and `Order` interfaces are provided; you write two implementations and two services. |

## Problem

A checkout takes a payment from a customer and a shipping calculator prices delivery from an order.

## Requirements

1. `SimpleWallet(initial)`: `initial >= 0` else `IllegalArgumentException`. `debit(amount)`: `amount <= 0` → `IllegalArgumentException`; more than the balance → `IllegalStateException` (balance unchanged).
2. `StandardCustomer(name, wallet)` implements `Customer`: `canAfford` is true when the wallet balance covers the amount; `pay` debits the wallet (`pay` beyond the funds → `IllegalStateException`, nothing debited). `getWallet()` returns the wallet.
3. `Cashier.checkout(customer, amount)`: if `customer.canAfford(amount)` → call `customer.pay(amount)`, add to revenue, return `"APPROVED"`; otherwise return `"DECLINED"` without calling `pay`. Null customer or `amount <= 0` → `IllegalArgumentException`. `revenue()` is the sum of approved amounts.
4. `ShippingCalculator.cost(order)`: free (0) when `order.itemTotal() >= 10000`; otherwise by `order.shippingCity()` (case-insensitive): Paris 1500, Lyon 1800, anything else 2500.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

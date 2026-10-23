# 37 · Abstraction (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 30 min | Level 2 — an `Account` base class and two subclasses as stubs; you write the bodies. |

## Problem

A bank has savings and checking accounts. Deposits, withdrawals and month-end processing follow the same steps for
every account, but two decisions differ: **may this withdrawal happen?** and **what is the monthly fee?**

## Requirements

1. `Account` owns the balance. Two decisions depend on the account type: `monthlyFee()` and `canWithdraw(long amount)`. The tests check with reflection that `Account` declares exactly these two abstract methods.
2. `deposit(amount)`: `amount > 0` or `IllegalArgumentException`.
3. `withdraw(amount)`: `amount > 0` or `IllegalArgumentException`. It asks `canWithdraw(amount)`; if the answer is no it throws `IllegalStateException` and the balance is unchanged.
4. `monthEnd()` subtracts `monthlyFee()` from the balance (even if that overdraws).
5. `SavingsAccount(minBalance)`: a withdrawal is allowed only if the balance afterwards is `>= minBalance`. Fee 0.
6. `CheckingAccount(overdraftLimit)`: allowed if the balance afterwards is `>= -overdraftLimit`. Fee 200, **waived while the balance is at least 100000**.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

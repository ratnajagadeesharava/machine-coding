# 37 · Abstraction (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 30 min | Level 2 — an abstract `Account` and two subclasses as stubs. Bodies throw `UnsupportedOperationException`. |

> **Concept:** an abstract class captures what is common (state + rules) and leaves the varying decisions as
> *abstract methods*. Callers see only "an account"; the details stay hidden in the subclasses.

## Problem

A bank has savings and checking accounts. Deposits, withdrawals and month-end processing follow the same steps for
every account, but two decisions differ: **may this withdrawal happen?** and **what is the monthly fee?**

## Requirements

1. `Account` is `abstract`, owns the balance, and has exactly two abstract methods: `monthlyFee()` and `canWithdraw(long amount)`. The tests check this with reflection.
2. `deposit(amount)`: `amount > 0` or `IllegalArgumentException`.
3. `withdraw(amount)`: `amount > 0` or `IllegalArgumentException`. It asks `canWithdraw(amount)`; if the answer is no it throws `IllegalStateException` and the balance is unchanged.
4. `monthEnd()` subtracts `monthlyFee()` from the balance (even if that overdraws).
5. `SavingsAccount(minBalance)`: a withdrawal is allowed only if the balance afterwards is `>= minBalance`. Fee 0.
6. `CheckingAccount(overdraftLimit)`: allowed if the balance afterwards is `>= -overdraftLimit`. Fee 200, **waived while the balance is at least 100000**.

## Constraints

- Deposit/withdraw/month-end logic is written once, in `Account`. The tests extend `Account` with their own tiny subclass and expect it to work.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

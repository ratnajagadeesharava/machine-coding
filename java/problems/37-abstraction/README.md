# 37 · Abstraction (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 30 min | Level 2 — an `Account` base class and two subclasses as stubs; you write the bodies. |

## Problem

You are building the account engine of a small bank. It offers savings and checking accounts. Depositing,
withdrawing and month-end processing follow the same steps for every account, but two decisions differ by account
type: **may this withdrawal happen?** and **what is the monthly fee?** Savings accounts must keep a minimum balance;
checking accounts may go into overdraft up to a limit and pay a monthly fee unless the customer is rich enough.

## Glossary

- **balance**: the account's current amount in whole cents (`long`); it can be negative on a checking account.
- **overdraft limit**: how far below zero a checking account may go.
- **minimum balance**: the lowest balance a savings account may be left with by a withdrawal.
- **month end**: the monthly processing step, `monthEnd()`, which charges the fee.

## Requirements

1. `Account` owns the balance (`balance()` starts at 0). Two decisions depend on the account type: `monthlyFee()` and `canWithdraw(long amount)`. The tests check with reflection that `Account` is abstract and declares exactly these two abstract methods; everything else works for any subclass that supplies just those two (the tests define such a subclass).
2. `deposit(amount)`: `amount > 0` or `IllegalArgumentException`. The balance increases by `amount`.
3. `withdraw(amount)`: `amount > 0` or `IllegalArgumentException`. It asks `canWithdraw(amount)` before every withdrawal; if the answer is no it throws `IllegalStateException` and the balance is unchanged. Otherwise the balance decreases by `amount`.
4. `monthEnd()` subtracts `monthlyFee()` from the balance (even if that overdraws, and even beyond an overdraft limit). Each call charges once.
5. `SavingsAccount(minBalance)`: a withdrawal is allowed only if the balance afterwards would be `>= minBalance`. Fee 0.
6. `CheckingAccount(overdraftLimit)`: a withdrawal is allowed if the balance afterwards would be `>= -overdraftLimit`. Fee 200, **waived while the balance is at least 100000** (at 99999 the fee is charged).

## Examples

```java
Account s = new SavingsAccount(1000);
s.deposit(1500);
s.withdraw(500);      // balance 1000, exactly the minimum: allowed
s.withdraw(1);        // would leave 999: IllegalStateException, balance still 1000
s.monthEnd();         // fee 0, balance 1000

Account c = new CheckingAccount(500);
c.deposit(100);
c.withdraw(600);      // balance -500, exactly the limit: allowed
c.withdraw(1);        // would leave -501: IllegalStateException
c.monthEnd();         // fee 200 -> balance -700

Account rich = new CheckingAccount(0);
rich.deposit(100_000);
rich.monthEnd();      // fee waived, balance 100000
```

A test subclass with fee 75 and deposit 100: two `monthEnd()` calls leave `100 - 75 - 75 = -50`.

## Edge cases

- `deposit(0)`, `deposit(-5)` and `withdraw(0)` throw `IllegalArgumentException` and leave the balance at its previous value.
- A declined withdrawal never changes the balance.
- The fee-waiver threshold is checked against the balance at the time of `monthEnd()`.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

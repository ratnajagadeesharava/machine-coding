# 37 · Abstraction (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 30 min | Level 2 — the `Account` trait, `AccountState` and two account types are stubs; `AccountError` is provided. |

## Problem

A bank has savings and checking accounts. Deposits, withdrawals and month-end processing follow the same steps for
every account, but two decisions differ: **may this withdrawal happen?** and **what is the monthly fee?**

## Requirements

1. The `Account` trait has `state()`, `state_mut()`, `monthly_fee()`, `can_withdraw(amount)` and the methods `balance`, `deposit`, `withdraw`, `month_end`.
2. `deposit(amount)`: `amount > 0` or `Err(AccountError::InvalidAmount)`.
3. `withdraw(amount)`: `amount > 0` or `Err(InvalidAmount)`. It asks `can_withdraw`; if no → `Err(AccountError::Declined)` with the balance unchanged.
4. `month_end()` subtracts `monthly_fee()` (even if that overdraws).
5. `SavingsAccount::new(min_balance)`: allowed only if the balance afterwards is `>= min_balance`. Fee 0.
6. `CheckingAccount::new(overdraft_limit)`: allowed if the balance afterwards is `>= -overdraft_limit`. Fee 200, **waived while the balance is at least 100000**.

7. The tests define their own `Configurable` type implementing only `state()`, `state_mut()`, `monthly_fee()` and `can_withdraw(amount)` and expect `balance`, `deposit`, `withdraw` and `month_end` to work for it.

## Running

```bash
cargo test -p p37_abstraction     # from rust/ — tests FAIL until you implement the stubs
cargo play run 37
```

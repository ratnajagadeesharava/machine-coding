# 37 · Abstraction (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 30 min | Level 2 — the `Account` trait, `AccountState` and two account types are stubs in `src/lib.rs`; `AccountError` is provided. |

> **Rust adaptation:** there are no abstract classes. The same idea is a **trait with required methods (the
> abstract part) and provided methods (the shared behaviour)**, plus a small state struct that implementors embed.

## Problem

A bank has savings and checking accounts. Deposits, withdrawals and month-end processing follow the same steps for
every account, but two decisions differ: **may this withdrawal happen?** and **what is the monthly fee?**

## Requirements

1. `Account` requires `state()`, `state_mut()`, `monthly_fee()` and `can_withdraw(amount)`. The shared methods `balance`, `deposit`, `withdraw`, `month_end` are **provided methods** you write once in the trait.
2. `deposit(amount)`: `amount > 0` or `Err(AccountError::InvalidAmount)`.
3. `withdraw(amount)`: `amount > 0` or `Err(InvalidAmount)`. It asks `can_withdraw`; if no → `Err(AccountError::Declined)` with the balance unchanged.
4. `month_end()` subtracts `monthly_fee()` (even if that overdraws).
5. `SavingsAccount::new(min_balance)`: allowed only if the balance afterwards is `>= min_balance`. Fee 0.
6. `CheckingAccount::new(overdraft_limit)`: allowed if the balance afterwards is `>= -overdraft_limit`. Fee 200, **waived while the balance is at least 100000**.

## Constraints

- The tests define their own `Configurable` implementor with just the four required methods and expect the provided methods to work for it.

## Running

```bash
cargo test -p p37_abstraction     # from rust/ — tests FAIL until you implement the stubs
cargo play run 37
```

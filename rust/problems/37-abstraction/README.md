# 37 · Abstraction (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 30 min | Level 2 — the `Account` trait, `AccountState` and two account types are stubs; `AccountError` is provided. |

## Problem

You are building the account engine of a small bank. It offers savings and checking accounts. Depositing,
withdrawing and month-end processing follow the same steps for every account, but two decisions differ by account
type: **may this withdrawal happen?** and **what is the monthly fee?** Savings accounts must keep a minimum balance;
checking accounts may go into overdraft up to a limit and pay a monthly fee unless the customer is rich enough.

## Glossary

- **balance**: the account's current amount in whole cents (`i64`); it can be negative on a checking account.
- **overdraft limit**: how far below zero a checking account may go.
- **minimum balance**: the lowest balance a savings account may be left with by a withdrawal.
- **month end**: the monthly processing step, `month_end()`, which charges the fee.

## Requirements

1. The `Account` trait has `state()`, `state_mut()`, `monthly_fee()`, `can_withdraw(amount)` and the methods `balance`, `deposit`, `withdraw`, `month_end`. `AccountState::new()` starts with balance 0.
2. `deposit(amount)`: `amount > 0` or `Err(AccountError::InvalidAmount)`. The balance increases by `amount`.
3. `withdraw(amount)`: `amount > 0` or `Err(InvalidAmount)`. It asks `can_withdraw` before every withdrawal; if no, `Err(AccountError::Declined)` with the balance unchanged. Otherwise the balance decreases by `amount`.
4. `month_end()` subtracts `monthly_fee()` from the balance (even if that overdraws, and even beyond an overdraft limit). Each call charges once.
5. `SavingsAccount::new(min_balance)`: a withdrawal is allowed only if the balance afterwards would be `>= min_balance`. Fee 0.
6. `CheckingAccount::new(overdraft_limit)`: a withdrawal is allowed if the balance afterwards would be `>= -overdraft_limit`. Fee 200, **waived while the balance is at least 100000** (at 99999 the fee is charged).
7. The tests define their own `Configurable` type implementing only `state()`, `state_mut()`, `monthly_fee()` and `can_withdraw(amount)` and expect `balance`, `deposit`, `withdraw` and `month_end` to work for it.

## Examples

```rust
let mut s = SavingsAccount::new(1000);
s.deposit(1500).unwrap();
s.withdraw(500).unwrap();   // balance 1000, exactly the minimum: allowed
s.withdraw(1);              // would leave 999: Err(AccountError::Declined), balance still 1000
s.month_end();              // fee 0, balance 1000

let mut c = CheckingAccount::new(500);
c.deposit(100).unwrap();
c.withdraw(600).unwrap();   // balance -500, exactly the limit: allowed
c.withdraw(1);              // would leave -501: Err(Declined)
c.month_end();              // fee 200 -> balance -700

let mut rich = CheckingAccount::new(0);
rich.deposit(100_000).unwrap();
rich.month_end();           // fee waived, balance 100000
```

A test account with fee 75 and deposit 100: two `month_end()` calls leave `100 - 75 - 75 = -50`.

## Edge cases

- `deposit(0)`, `deposit(-5)` and `withdraw(0)` return `Err(InvalidAmount)` and leave the balance unchanged.
- A declined withdrawal never changes the balance.
- The fee-waiver threshold is checked against the balance at the time of `month_end()`.

## Running

```bash
cargo test -p p37_abstraction     # from rust/ — tests FAIL until you implement the stubs
cargo play run 37
```

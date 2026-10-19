# 75 · Bank Transfers Without Deadlock (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 45 min | Level 3 — infrastructure provided. Test utilities for threads/clocks exist; business/domain code is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Build the core of an in-memory bank: accounts with balances and an atomic `transfer` between two accounts that
is called by thousands of threads at once. Money must never be created, destroyed or temporarily "in flight", an account
must never go negative, and the system must never deadlock - even when A sends to B at the very moment B sends to A.
An auditor thread may ask for the total of all balances at any time; it must always see the true total.

## Requirements

1. `open(account, balance)` creates an account (names are unique, initial balance >= 0).
2. `transfer(from, to, amount)` moves `amount` atomically: either both balances change or neither does.
3. A transfer from an account that has less than `amount` fails (returns `false`) and changes nothing. Transferring exactly the balance is allowed.
4. Invalid requests are rejected without side effects: unknown account, `from == to`, `amount <= 0`.
5. `balance(account)` returns the current balance.
6. `total()` returns the sum of all balances and must **never** observe a half-applied transfer.

## Constraints and thread-safety requirements

- Any number of threads call `transfer`, `balance` and `total` concurrently.
- No deadlock for any pattern of transfers: pairs sending to each other, cycles A->B->C->A, one hot account receiving from everybody.
- Two transfers between disjoint pairs of accounts should not have to wait for each other.
- Throughput: ~16 threads x 10,000 transfers over 20 accounts well within a few seconds.
- Accounts are opened before the concurrent phase; `open` need not be concurrent with transfers (but must not corrupt anything if it is).

## Examples

```
A=100, B=100
thread 1: transfer(A, B, 30)      thread 2: transfer(B, A, 10)     (at the same instant)
-> both succeed, A=80, B=120, total() is 200 at every moment

A=500, 100 threads each: transfer(A, sink, 10)
-> exactly 50 succeed, A=0, sink=500
```

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: invalid requests throw `IllegalArgumentException`. Rust: `Err(BankError::UnknownAccount | AccountExists | InvalidRequest)`.
- `transfer` returns `false` / `Ok(false)` for insufficient funds - not an error.
- `total()` is a consistent snapshot: at any time, `total()` equals the sum of the opening balances (the tests never deposit or withdraw outside of transfers).
- Balances are `long` (Java) / `u64` (Rust). Overflow is out of scope.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

#[derive(Debug, PartialEq, Eq)]
pub enum BankError {
    UnknownAccount,
    AccountExists,
    /// amount == 0, or from == to
    InvalidRequest,
}

pub trait Bank: Send + Sync {
    /// Creates an account. `Err(AccountExists)` if the name is taken.
    fn open(&self, account: &str, balance: u64) -> Result<(), BankError>;

    fn balance(&self, account: &str) -> Result<u64, BankError>;

    /// Moves money atomically: either both balances change or neither does. `Ok(false)` (nothing changes) when
    /// `from` has less than `amount`; a transfer of exactly the balance succeeds.
    /// `Err(UnknownAccount)` for a missing account, `Err(InvalidRequest)` for amount == 0 or from == to.
    fn transfer(&self, from: &str, to: &str, amount: u64) -> Result<bool, BankError>;

    /// Sum of all balances. Must never include a half-applied transfer (no money "in flight").
    fn total(&self) -> u64;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create() -> Option<Arc<dyn Bank>> { ... }
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Concurrency tests run many threads against one object, use deadlock timeouts (a hang becomes a failure) and repeat a workload 100 times to catch rare interleavings.

- A <-> B ping-pong by 4 threads (the textbook deadlock) and an 8-account ring where each thread sends to the next.
- 100 threads withdrawing 10 from a 500 account: exactly 50 succeed (no double spend, no lost update).
- 16 threads of random transfers: total conserved, no negative balance.
- An auditor calls `total()` in a tight loop while 6 threads move money: it must never see a wrong total *and* must make progress.
- **Repeat 100x** a small deadlock-prone workload; every run has a deadlock timeout.

## Language notes

`Mutex<u64>` per account inside an `Arc`, ordered by an id field. Keep the guards alive until the update is done; mind which guard is `from` and which is `to` after you ordered them.

## Running

```bash
cargo test -p p75_bank_transfer
STRICT=1 cargo test -p p75_bank_transfer   # fail instead of skip while unimplemented
cargo test -p p75_bank_transfer --features hidden   # interviewer scenarios
cargo play run 75              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`run_concurrently`, `eventually`, `stays`, `watchdog`, `sut!`): `rust/support`. `run_concurrently` aborts the test run with a `TIMEOUT` message after 60 s, so a deadlock fails instead of freezing `cargo test`.

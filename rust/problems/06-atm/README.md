# 06 · ATM (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 60 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Implement the core of an ATM: authenticate a card holder, let them check their balance, withdraw cash from the
machine's note cassettes, and deposit money into their account.

## Requirements

1. Accounts are identified by **card id** and protected by a **PIN**; each has a balance (in whole currency units).
2. The machine holds notes in denominations **10, 20, 50 and 100**; the operator loads them.
3. `authenticate(card, pin)` starts a **session** and returns its token. Three consecutive wrong PINs **lock** the card: it can no longer authenticate, even with the right PIN. A correct PIN resets the failure count.
4. Within a session the customer can query the **balance**, **withdraw**, and **deposit**. After `endSession` the token is invalid.
5. A withdrawal amount must be a positive multiple of 10, not exceed the account balance, and be payable **exactly** from the notes currently in the machine, using the **fewest notes** possible.
6. A session may withdraw at most **500** in total.
7. A withdrawal is atomic: either the account is debited **and** the notes leave the machine, or nothing changes.
8. A deposit must be positive and increases the account balance. (Deposited cash does not become dispensable in this version.)

## Constraints

- Notes are limited: choosing the fewest notes is a **bounded** problem, and the greedy choice can fail (see the example).
- All failures are plain refusals (empty/false); no exceptions are required by the contract.

## Examples

```
machine holds: 50×1, 20×3 (no 10s)
withdraw(60) → {20: 3}    (greedy would take the 50 and then fail)
withdraw(30) → refused    (no way to pay exactly 30)
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `authenticate` returns a non-empty session token on success; empty for unknown cards, wrong PINs, and locked cards. Locking happens on the third consecutive wrong PIN.
- `balance(session)`, `withdraw`, `deposit` on an unknown or ended session are refused (`empty`/`false`).
- `withdraw` returns the notes dispensed as a map `denomination → count` whose total equals the amount, or empty when refused (invalid amount, insufficient balance, session limit exceeded, machine cannot pay exactly).
- The session limit counts only successful withdrawals.
- `cashInMachine()` is the total value of notes currently in the machine.

## Test entry point (Rust)

The tests drive your system through the thin, primitive-typed contract below (`src/contract.rs`). It is
**not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up
in `src/lib.rs`.

```rust
//! Thin test entry point. Your domain model lives behind it.

use std::collections::BTreeMap;

pub trait AtmApi {
    fn add_account(&mut self, card_id: &str, pin: &str, balance: u64);

    /// Adds notes of the given denomination (10, 20, 50 or 100) to the machine.
    fn load_cash(&mut self, denomination: u32, count: u32);

    /// Returns a session token, or `None` (unknown card, wrong PIN, locked card).
    fn authenticate(&mut self, card_id: &str, pin: &str) -> Option<String>;

    fn balance(&self, session: &str) -> Option<u64>;

    /// Notes dispensed (denomination -> count), or `None` if refused.
    fn withdraw(&mut self, session: &str, amount: u32) -> Option<BTreeMap<u32, u32>>;

    fn deposit(&mut self, session: &str, amount: u64) -> bool;

    fn end_session(&mut self, session: &str);

    fn cash_in_machine(&self) -> u64;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create() -> Option<Box<dyn AtmApi>> {
    None
}
```

## Language notes

Everything is `&mut self`. The note map is a `BTreeMap<u32, u32>` (denomination → count) so ordering is deterministic.

## Running

```bash
cargo test -p p06_atm
STRICT=1 cargo test -p p06_atm   # fail instead of skip while unimplemented
cargo test -p p06_atm --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p06_atm --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

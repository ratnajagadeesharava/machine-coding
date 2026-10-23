# 16 · Digital Wallet (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 90 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building the core of a digital wallet service, like the balance behind a payments app. Each customer has a
wallet that holds money; they can deposit, withdraw, and send money to another wallet. Mobile clients retry requests
over flaky networks, so a request may reach the server twice. To stay safe, every money-moving request carries a
client-chosen **idempotency key**, and the service must never apply the same key twice. Finance also wants a
per-wallet **statement** listing every change to a wallet. Many requests arrive at the same time from many threads.

## Glossary

- **Wallet**: a named account holding a non-negative balance.
- **Minor unit**: the smallest money unit (cents). All amounts and balances are whole numbers of these.
- **Credit / debit / transfer**: deposit into a wallet / withdrawal from a wallet / move money from one wallet to another.
- **Idempotency key**: a client-chosen string identifying one logical request, so that a retry can be recognised and ignored.
- **Statement**: the history of signed changes to one wallet.
- **Source / destination**: the wallet money leaves / arrives in. A debit has a source only, a credit a destination only.

## Requirements

1. A **wallet** is identified by a name and starts with a balance of `0`. Wallets are created explicitly (`create_wallet`); operations on a wallet that was never created are refused.
2. Amounts are whole **minor units** (cents), always positive. Never use floating point.
3. Three money operations: **credit** (deposit into a wallet), **debit** (withdraw from a wallet) and **transfer** (move money from one wallet to another).
4. A debit or transfer that would take the source below zero is refused (`INSUFFICIENT_FUNDS`). Spending a balance down to exactly `0` is allowed. Balances never go negative.
5. A transfer is **atomic**: either both wallets change or neither does. Money is never created or destroyed.
6. Every money operation carries an **idempotency key**. Repeating a key that was already applied must not change anything and reports a duplicate.
7. Each wallet keeps a **statement**: the signed entries of every applied operation touching it, in the order they were applied. For every wallet the sum of its statement entries equals its balance.
8. Many threads use the service at once. In particular, transfers in opposite directions between the same two wallets must not deadlock, and concurrent withdrawals must never overdraw. If several threads submit the same key at once, exactly one of them gets `APPLIED` and all others get `DUPLICATE`.

## Constraints

- Thread-safe.
- Unrelated wallets should not block each other more than necessary.
- A refused or duplicate request leaves no trace (balances, statements).

## Examples

Basic flow with a retry:

```
create_wallet("alice"), create_wallet("bob")
credit("k1", "alice", 1000) -> APPLIED       credit("k1", "alice", 1000) -> DUPLICATE (balance is still 1000)
transfer("k2", "alice", "bob", 300) -> APPLIED     alice 700, bob 300
debit("k3", "bob", 500) -> INSUFFICIENT_FUNDS      statement(bob) = ["+300"]
```

Statements (refused and duplicate requests leave no entry):

```
credit("k1", "alice", 1000)         alice: +1000
transfer("k2", "alice", "bob", 300) alice: -300          bob: +300
debit("k3", "alice", 50)            alice: -50
transfer("k4", "bob", "alice", 100) alice: +100          bob: -100
debit("k5", "bob", 9999)            -> INSUFFICIENT_FUNDS (no entry)
credit("k1", "alice", 5)            -> DUPLICATE          (no entry)
statement(alice) = ["+1000", "-300", "-50", "+100"]      statement(bob) = ["+300", "-100"]
```

A refused request does not use up its key:

```
debit("k1", "alice", 50)  -> INSUFFICIENT_FUNDS   (alice has 0)
credit("k2", "alice", 100) -> APPLIED
debit("k1", "alice", 50)  -> APPLIED              (k1 was never consumed; balance 50)
```

A key is global, whatever else changes:

```
credit("k1", "alice", 100) -> APPLIED
credit("k1", "alice", 999) / credit("k1", "bob", 100) / debit("k1", "alice", 10) / transfer("k1", "alice", "bob", 10)
                           -> all DUPLICATE, nothing changes
```

Concurrency: with a balance of 100, 100 threads each doing `debit("d<i>", "alice", 3)` -> exactly 33 `APPLIED`, 67 `INSUFFICIENT_FUNDS`, balance 1.

## Edge cases

- `transfer(k, "ghost", "ghost", 5)` or `transfer(k, "alice", "alice", 5)` -> `INVALID_AMOUNT` (same source and destination is checked before wallet existence and balance).
- `credit(k, "ghost", 0)` -> `INVALID_AMOUNT` (amount is checked before wallet existence).
- `transfer(k, "alice", "ghost", 10)` and `transfer(k, "ghost", "alice", 10)` -> `UNKNOWN_WALLET`; nothing changes for alice.
- `create_wallet("")`, `create_wallet(" ")` -> `false`; creating an existing id again -> `false` and its balance and statement are untouched.
- `balance("nobody")` is empty; `statement("nobody")` is an empty list.
- Changing the returned statement list afterwards does not affect the wallet (it is a snapshot).

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `create_wallet(id)` returns `true` if a new wallet was created and `false` if `id` is empty/blank or already exists (the existing wallet is untouched).
- `credit`, `debit` and `transfer` return exactly one of these strings: `APPLIED`, `DUPLICATE`, `INSUFFICIENT_FUNDS`, `UNKNOWN_WALLET`, `INVALID_AMOUNT`. (`credit` can never return `INSUFFICIENT_FUNDS`.)
- Checks are made in this order, and the first failing one decides the result: (1) the key was already applied -> `DUPLICATE`; (2) `amount <= 0`, or a transfer whose source and destination are the same wallet -> `INVALID_AMOUNT`; (3) a wallet involved does not exist -> `UNKNOWN_WALLET`; (4) the source would go below zero -> `INSUFFICIENT_FUNDS`.
- Idempotency keys are **global**: one namespace shared by all wallets and all three operation kinds. A key is consumed **only** by an `APPLIED` operation; a refused request does not consume its key and may be retried with the same key.
- A repeated key is a `DUPLICATE` **whatever its other parameters are** (different wallet, amount or operation kind): nothing is applied and the original operation is not modified.
- `balance(id)` is the balance in minor units, empty for an unknown wallet.
- `statement(id)` lists entries oldest first; every entry is the signed amount with an explicit sign, for example `+100` or `-30`. A transfer adds `-amount` to the source's statement and `+amount` to the destination's. Refused and duplicate requests add nothing. Unknown wallet -> empty list. The returned list is a snapshot.

## Test entry point (Rust)

The tests drive your system through the thin, primitive-typed contract below (`src/contract.rs`). It is
**not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up
in `src/lib.rs`.

```rust
//! Thin test entry point. Your domain model lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).

pub trait WalletApi: Send + Sync {
    /// Creates an empty wallet. False if the id is blank or already exists.
    fn create_wallet(&self, wallet_id: &str) -> bool;

    /// APPLIED, DUPLICATE, INVALID_AMOUNT or UNKNOWN_WALLET.
    fn credit(&self, key: &str, wallet_id: &str, amount: i64) -> String;

    /// APPLIED, DUPLICATE, INVALID_AMOUNT, UNKNOWN_WALLET or INSUFFICIENT_FUNDS.
    fn debit(&self, key: &str, wallet_id: &str, amount: i64) -> String;

    /// APPLIED, DUPLICATE, INVALID_AMOUNT, UNKNOWN_WALLET or INSUFFICIENT_FUNDS.
    fn transfer(&self, key: &str, from_wallet_id: &str, to_wallet_id: &str, amount: i64) -> String;

    fn balance(&self, wallet_id: &str) -> Option<i64>;

    /// Signed entries such as "+100" and "-30", oldest first. Empty for unknown wallets.
    fn statement(&self, wallet_id: &str) -> Vec<String>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create() -> Option<Arc<dyn WalletApi>> {
    None
}
```

## Language notes

The trait is `&self` + `Send + Sync`. Amounts and balances are `i64` minor units. `balance` returns `Option<i64>` (`None` for an unknown wallet).

## Running

```bash
cargo test -p p16_wallet
STRICT=1 cargo test -p p16_wallet   # fail instead of skip while unimplemented
cargo test -p p16_wallet --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p16_wallet --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

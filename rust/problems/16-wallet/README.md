# 16 · Digital Wallet (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 90 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Build the core of a digital wallet service: accounts holding money, deposits, withdrawals and transfers between
accounts. Clients retry requests over flaky networks, so every money-moving request carries a client-chosen
**idempotency key** and must never be applied twice. Finance also wants a per-wallet statement.

## Requirements

1. A **wallet** is identified by a name and starts with a balance of `0`. Wallets are created explicitly.
2. Amounts are whole **minor units** (cents), always positive. Never use floating point.
3. Three money operations: **credit** (deposit into a wallet), **debit** (withdraw from a wallet) and **transfer** (move money from one wallet to another).
4. A debit or transfer that would take the source below zero is refused. Balances never go negative.
5. A transfer is **atomic**: either both wallets change or neither does. Money is never created or destroyed.
6. Every money operation carries an **idempotency key**. Repeating a key that was already applied must not change anything and reports a duplicate.
7. Each wallet keeps a **statement**: the signed entries of every applied operation touching it, in the order they were applied.
8. Many threads use the service at once. In particular, transfers in opposite directions between the same two wallets must not deadlock, and concurrent withdrawals must never overdraw.

## Constraints

- Thread-safe.
- Unrelated wallets should not block each other more than necessary.
- A refused or duplicate request leaves no trace (balances, statements).

## Examples

```
createWallet("alice"), createWallet("bob")
credit("k1", "alice", 1000) → APPLIED       credit("k1", "alice", 1000) → DUPLICATE (balance is still 1000)
transfer("k2", "alice", "bob", 300) → APPLIED     alice 700, bob 300
debit("k3", "bob", 500) → INSUFFICIENT_FUNDS      statement(bob) = ["+300"]
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `createWallet(id)` returns `true` if a new wallet was created and `false` if `id` is empty/blank or already exists (the existing wallet is untouched).
- `credit`, `debit` and `transfer` return exactly one of these strings: `APPLIED`, `DUPLICATE`, `INSUFFICIENT_FUNDS`, `UNKNOWN_WALLET`, `INVALID_AMOUNT`.
- Checks are made in this order, and the first failing one decides the result: (1) the key was already applied → `DUPLICATE`; (2) `amount <= 0`, or a transfer whose source and destination are the same wallet → `INVALID_AMOUNT`; (3) a wallet involved does not exist → `UNKNOWN_WALLET`; (4) the source would go below zero → `INSUFFICIENT_FUNDS`.
- Idempotency keys are **global**: one namespace shared by all wallets and all three operation kinds. A key is consumed **only** by an `APPLIED` operation; a refused request does not consume its key and may be retried with the same key.
- A repeated key is a `DUPLICATE` **whatever its other parameters are** (different wallet, amount or operation kind): nothing is applied and the original operation is not modified.
- `balance(id)` is the balance in minor units, empty for an unknown wallet.
- `statement(id)` lists entries oldest first; every entry is the signed amount with an explicit sign, for example `+100` or `-30`. A transfer adds `-amount` to the source's statement and `+amount` to the destination's. Refused and duplicate requests add nothing. Unknown wallet → empty list. The returned list is a snapshot.

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

The trait is `&self` + `Send + Sync`. Amounts and balances are `i64` minor units. Interior mutability (`Mutex`/`RwLock`) is yours to design.

## Running

```bash
cargo test -p p16_wallet
STRICT=1 cargo test -p p16_wallet   # fail instead of skip while unimplemented
cargo test -p p16_wallet --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p16_wallet --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

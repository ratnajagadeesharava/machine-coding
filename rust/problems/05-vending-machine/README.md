# 05 · Vending Machine (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 60 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Build the brain of a coin-operated vending machine: customers insert coins, pick a slot, receive a product and
change, or cancel and get their money back.

## Requirements

1. The machine has product **slots** (e.g. `A1`). An operator loads a slot with a product name, a price (in cents) and a quantity.
2. The operator can load the machine's **coin inventory** (denominations 5, 10, 25, 100).
3. A customer inserts coins; the machine tracks their **balance**. Only denominations 5, 10, 25 and 100 are accepted; anything else is rejected and the balance is unchanged.
4. Selecting a slot attempts a purchase. Rules are checked in this order: invalid slot → sold out → insufficient funds → cannot make change.
5. A successful purchase decrements the slot's quantity, adds the customer's coins to the coin inventory, resets the balance to 0 and returns **change** using the **fewest possible coins** from the inventory.
6. If exact change cannot be made, the purchase is refused and nothing changes (the customer can cancel).
7. **Cancel** returns exactly the coins the customer inserted (not equivalent coins from inventory) and resets the balance to 0.
8. Returned coins (change or refund) land in a **tray**; the customer collects them with `takeReturnedCoins`, which empties the tray.

## Constraints

- Inventory counts are non-negative integers; a purchase must be atomic with respect to quantity, balance and coin inventory.
- Behavior depends on what happened before (balance, tray, quantities) — think about what states the machine can be in and which operations are legal in each.

## Examples

```
loadProduct(A1, "Chips", 65, 2); loadCoins(25, 4); loadCoins(10, 4); loadCoins(5, 4)
insertCoin(100) → true     balance() = 100
select(A1) → "DISPENSED"   takeReturnedCoins() = [25, 10]   balance() = 0
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `select` returns exactly one of `DISPENSED`, `INVALID_SLOT`, `SOLD_OUT`, `INSUFFICIENT_FUNDS`, `CANNOT_MAKE_CHANGE`.
- Change is computed from the inventory *after* adding the customer's inserted coins. If no combination of available coins equals the change, the result is `CANNOT_MAKE_CHANGE`.
- `takeReturnedCoins` returns the tray content in descending order and clears it. Change and refunds both use the tray.
- `cancel` returns exactly the inserted coins (descending order) to the tray; with nothing inserted it does nothing.
- `insertCoin` returns `false` for unsupported denominations.
- `stockOf(slot)` is the remaining quantity, 0 for unknown slots.

## Test entry point (Rust)

The tests drive your system through the thin, primitive-typed contract below (`src/contract.rs`). It is
**not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up
in `src/lib.rs`.

```rust
//! Thin test entry point. Your domain model lives behind it.

pub trait VendingMachineApi {
    fn load_product(&mut self, slot: &str, name: &str, price_cents: u32, quantity: u32);

    /// Adds coins of the given denomination (5, 10, 25 or 100) to the machine's inventory.
    fn load_coins(&mut self, denomination: u32, count: u32);

    /// Returns false for unsupported denominations.
    fn insert_coin(&mut self, denomination: u32) -> bool;

    /// DISPENSED, INVALID_SLOT, SOLD_OUT, INSUFFICIENT_FUNDS or CANNOT_MAKE_CHANGE.
    fn select(&mut self, slot: &str) -> String;

    /// Refunds the inserted coins into the tray.
    fn cancel(&mut self);

    /// Credit currently inserted.
    fn balance(&self) -> u32;

    /// Empties the tray; coins in descending order.
    fn take_returned_coins(&mut self) -> Vec<u32>;

    fn stock_of(&self, slot: &str) -> u32;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create() -> Option<Box<dyn VendingMachineApi>> {
    None
}
```

## Language notes

Everything is `&mut self`. Consider enums for outcomes and for machine state, and think about whether the denominations belong in a type.

## Running

```bash
cargo test -p p05_vending_machine
STRICT=1 cargo test -p p05_vending_machine   # fail instead of skip while unimplemented
cargo test -p p05_vending_machine --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p05_vending_machine --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

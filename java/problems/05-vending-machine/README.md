# 05 · Vending Machine (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 60 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

**What you are building.** The control logic of a coin-operated snack/drink vending machine. An operator loads product **slots** (like `A1`: chips, 65 cents, 2 units) and puts coins into the machine's change float. A customer inserts coins one at a time, the machine shows their running **balance**, and they press a slot's button. The machine either hands over the product and gives back **change**, or refuses and explains why (bad slot, sold out, not enough money, or it cannot give exact change). The customer can also cancel and get their own coins back. Money the machine gives back falls into a **tray**, which the customer empties by taking the coins.

## Requirements

1. The machine has product **slots** (e.g. `A1`). An operator loads a slot with a product name, a price (in cents) and a quantity (`loadProduct(slot, name, priceCents, quantity)`). All money amounts are integer cents.
2. The operator can load the machine's **coin inventory** (`loadCoins(denomination, count)`, denominations 5, 10, 25, 100). This is the float used to make change; it starts empty.
3. A customer inserts coins (`insertCoin(denomination)`, returns `true` when accepted); the machine tracks their **balance** (the sum of the inserted coins not yet spent or refunded). Only denominations 5, 10, 25 and 100 are accepted; anything else is rejected and the balance is unchanged.
4. Selecting a slot attempts a purchase. Rules are checked in this order: invalid slot → sold out → insufficient funds → cannot make change.
5. A successful purchase decrements the slot's quantity, adds the customer's coins to the coin inventory, resets the balance to 0 and returns **change** using the **fewest possible coins** from the inventory.
6. If exact change cannot be made, the purchase is refused (`CANNOT_MAKE_CHANGE`) and nothing changes: balance, inserted coins, quantity, coin inventory and tray stay as they were, so the customer can cancel or insert other coins. Any other refusal likewise changes nothing.
7. **Cancel** returns exactly the coins the customer inserted (not equivalent coins from inventory) and resets the balance to 0.
8. Returned coins (change or refund) land in a **tray**; the customer collects them with `takeReturnedCoins`, which empties the tray.

## Constraints

- Inventory counts are non-negative integers; a purchase must be atomic with respect to quantity, balance and coin inventory.
- Behavior depends on what happened before (balance, tray, quantities) — think about what states the machine can be in and which operations are legal in each.

## Examples

Setup used by examples 1-4: `loadProduct(A1, "Chips", 65, 2)`, `loadProduct(B1, "Soda", 100, 1)`, and 4 coins each of 25, 10 and 5 loaded.

**1. Purchase with change**

```
insertCoin(100)        -> true      balance() = 100
select("A1")           -> "DISPENSED"
takeReturnedCoins()    -> [25, 10]  (change 35 with the fewest coins)
takeReturnedCoins()    -> []        (the tray was emptied)
balance() = 0          stockOf("A1") = 1
```

**2. Not enough money, then more coins**

```
insertCoin(25); select("A1") -> "INSUFFICIENT_FUNDS"   balance() = 25, stockOf("A1") = 2
insertCoin(25); insertCoin(25)                          balance() = 75
select("A1")           -> "DISPENSED"
takeReturnedCoins()    -> [10]      (75 - 65)
```

**3. Cancel returns the very coins inserted**

```
insertCoin(25); insertCoin(100); insertCoin(5)
cancel()               balance() = 0
takeReturnedCoins()    -> [100, 25, 5]   (descending)
```

**4. Other refusals**

```
insertCoin(3) -> false; insertCoin(50) -> false     balance() = 0
insertCoin(100); select("Z9") -> "INVALID_SLOT"     balance() = 100
select("B1") -> "DISPENSED" (exact, tray empty); insertCoin(100); select("B1") -> "SOLD_OUT"   balance() = 100
```

**5. Cannot make change** (a fresh machine, coin inventory empty): `loadProduct(A1, "Chips", 65, 1)`, `insertCoin(100)`. Change of 35 is owed but the inventory only contains the customer's 100-coin, so `select("A1")` -> `"CANNOT_MAKE_CHANGE"`; balance stays 100, `stockOf("A1")` stays 1, tray is empty. With product B1 at 100 and four 25-coins inserted into an empty inventory, `select("B1")` -> `"DISPENSED"` (exact payment needs no change).

**6. Fewest coins** (Gum 10 cents, inventory 2x25, 5x10, 5x5; customer inserts a 25): change is 15, returned as `[10, 5]` (two coins), not `[5, 5, 5]`.

## Edge cases

- Check order when several things are wrong: invalid slot, then sold out, then insufficient funds, then cannot make change.
- Rejected coins never change the balance; `cancel` with nothing inserted leaves the tray unchanged.
- Quantity 0 means sold out; `stockOf` of an unknown slot is 0.
- Change is made from the inventory plus the coins the customer just inserted, because those coins join the inventory when the purchase succeeds.
- Coins in the tray (change and refunds) stay there across operations until `takeReturnedCoins` is called; the next call returns all of them in descending order.
- Each `loadCoins` call adds to the inventory; repeated loads of one denomination accumulate.

## Glossary

- **Slot**: a named position (e.g. `A1`) holding one product at one price with a remaining quantity.
- **Balance**: cents currently credited to the customer.
- **Coin inventory**: the machine's own store of coins by denomination, used to give change.
- **Tray**: where returned coins (change or cancel refund) wait until `takeReturnedCoins`.
- **Change**: balance minus price, given back in coins.

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `select` returns exactly one of `DISPENSED`, `INVALID_SLOT`, `SOLD_OUT`, `INSUFFICIENT_FUNDS`, `CANNOT_MAKE_CHANGE`.
- Change is computed from the inventory *after* adding the customer's inserted coins. If no combination of available coins equals the change, the result is `CANNOT_MAKE_CHANGE`.
- `takeReturnedCoins` returns the tray content in descending order and clears it. Change and refunds both use the tray.
- `cancel` returns exactly the inserted coins (descending order) to the tray; with nothing inserted it does nothing.
- `insertCoin` returns `false` for unsupported denominations.
- `stockOf(slot)` is the remaining quantity, 0 for unknown slots.
- Change is the whole balance minus the price (not just the last coin inserted), and it is given as coins of the four denominations only.
- After a successful `select` the customer's inserted coins are part of the coin inventory and the balance is 0; after `cancel` they are not added to the inventory.

## Test entry point (Java)

The tests drive your system through the thin, primitive-typed contract below. It is **not** your domain
model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in
`solution/Solution.java`.

```java
package contract;

import java.util.List;

/** Thin test entry point. Your domain model lives behind it. */
public interface VendingMachineApi {

    void loadProduct(String slot, String name, int priceCents, int quantity);

    /** Adds coins of the given denomination (5, 10, 25 or 100) to the machine's inventory. */
    void loadCoins(int denomination, int count);

    /** Returns false for unsupported denominations. */
    boolean insertCoin(int denomination);

    /** DISPENSED, INVALID_SLOT, SOLD_OUT, INSUFFICIENT_FUNDS or CANNOT_MAKE_CHANGE. */
    String select(String slot);

    /** Refunds the inserted coins into the tray. */
    void cancel();

    /** Credit currently inserted. */
    int balance();

    /** Empties the tray; coins in descending order. */
    List<Integer> takeReturnedCoins();

    int stockOf(String slot);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    VendingMachineApi create()
}
```

## Language notes

Money is `int` cents. The tray is observable only through `takeReturnedCoins`.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/05-vending-machine test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

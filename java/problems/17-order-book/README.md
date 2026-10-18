# 17 · Stock Exchange Order Book (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 120 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Build the matching engine of a stock exchange for a **single instrument**. Traders submit limit orders to buy or
sell; the engine pairs compatible orders into trades immediately and keeps the unmatched rest in an order book
that traders can inspect. Real exchanges process millions of orders a day, so the engine has to stay fast as the
book grows.

## Requirements

1. An order is a **side** (buy or sell), a limit **price** and a **quantity**, all whole numbers. Every accepted order gets a unique id.
2. A buy order may trade with sell orders priced **at or below** its limit; a sell order may trade with buy orders priced **at or above** its limit.
3. **Price-time priority**: an incoming order trades first with the best-priced opposite orders (lowest ask for a buyer, highest bid for a seller); among orders at the same price the **earliest** submitted trades first.
4. Each trade executes at the **price of the resting order** (the one that was already in the book), for the smaller of the two remaining quantities.
5. An incoming order keeps trading until it is filled or no compatible order remains. Whatever is left **rests in the book** at its limit price. Partial fills are normal.
6. A resting order can be **cancelled**. A partially filled order can be cancelled too; only its remaining quantity is withdrawn.
7. Observers can ask for the best bid and best ask, the total resting quantity at a price level, and how much of a given order is still open.
8. Every trade is recorded in execution order.

## Constraints

- Single-threaded access.
- Handle on the order of 200,000 orders and cancels with prices spread over tens of thousands of levels: no operation may scan the whole book.
- Only trade when prices cross: the best bid is always strictly below the best ask after any operation.

## Examples

```
SELL 5 @ 100 rests (id 1)
BUY  8 @ 105  → trades 5 @ 100 with order 1, then rests with 3 left at 105   (trade "2:1:100:5")
BUY  1 @ 99   rests behind it.         bestBid = 105, bestAsk = none, volumeAt(BUY, 105) = 3
```

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- Sides are exactly `"BUY"` or `"SELL"` (upper case). `submit` returns the new order id, or empty (and changes nothing) when the side is not one of these, or the price or quantity is `<= 0`. Ids are unique across both sides.
- `cancel(id)` returns `true` if the order was resting with a positive remaining quantity (it is removed), and `false` for unknown, fully filled or already cancelled ids.
- `bestBid()` / `bestAsk()` return the price of the best resting buy / sell level, empty when that side is empty.
- `volumeAt(side, price)` is the sum of remaining quantities of resting orders of that side at exactly that price; `0` if none or if the side is not valid.
- `remaining(id)` is the unfilled quantity of the order; `0` for a filled or cancelled order; empty for an id that was never issued.
- `trades()` returns one string per trade, oldest first, of the form `buyId:sellId:price:quantity` (for example `2:1:100:5`) — the id of the buy order first, whichever side was incoming. It is a snapshot.

## Test entry point (Java)

The tests drive your system through the thin, primitive-typed contract below. It is **not** your domain
model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in
`solution/Solution.java`.

```java
package contract;

import java.util.List;
import java.util.OptionalLong;

/** Thin test entry point. Your domain model lives behind it. */
public interface OrderBookApi {

    /** side is "BUY" or "SELL". Returns the order id, or empty if the request is invalid. */
    OptionalLong submit(String side, long price, long quantity);

    /** True if the order was resting with quantity left and has been removed. */
    boolean cancel(long orderId);

    OptionalLong bestBid();

    OptionalLong bestAsk();

    /** Total remaining quantity resting at exactly this price on that side. */
    long volumeAt(String side, long price);

    /** Unfilled quantity; 0 if filled or cancelled; empty if the id was never issued. */
    OptionalLong remaining(long orderId);

    /** "buyId:sellId:price:quantity", oldest first. */
    List<String> trades();
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    OrderBookApi create()
}
```

## Language notes

Prices and quantities are `long`. Order ids are opaque positive `long`s. Single-threaded.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/17-order-book test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

# 17 · Stock Exchange Order Book (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 120 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building the matching engine of a stock exchange for a **single instrument** (one share symbol). Traders
submit limit orders: "buy 10 shares at no more than 99" or "sell 5 shares at no less than 101". The engine pairs
compatible buy and sell orders into trades immediately, and keeps the unmatched rest in an **order book** that
traders can inspect (best prices, how much is on offer at a price). Real exchanges process millions of orders a day,
so the engine has to stay fast as the book grows.

## Glossary

- **Buy order (bid)** / **sell order (ask)**: an offer to buy / sell at a limit price or better.
- **Limit price**: the worst price the trader accepts (highest for a buyer, lowest for a seller).
- **Incoming order**: the order being submitted right now. **Resting order**: an order already waiting in the book.
- **Price level**: all resting orders of one side at exactly one price.
- **Best bid** / **best ask**: the highest resting buy price / the lowest resting sell price.
- **Crossing**: a buy and a sell cross when the buy price is at least the sell price; crossing orders trade.
- **Fill / partial fill**: an order trades its whole quantity / only part of it, leaving some quantity open.
- **Remaining quantity**: the quantity of an order that is not yet traded (and not cancelled).

## Requirements

1. An order is a **side** (buy or sell), a limit **price** and a **quantity**, all whole numbers. Every accepted order gets a unique id (ids are unique across both sides).
2. A buy order may trade with sell orders priced **at or below** its limit; a sell order may trade with buy orders priced **at or above** its limit. Equal prices trade.
3. **Price-time priority**: an incoming order trades first with the best-priced opposite orders (lowest ask for a buyer, highest bid for a seller); among orders at the same price the **earliest** submitted trades first. A resting order that was partially filled keeps its place in line.
4. Each trade executes at the **price of the resting order** (the one that was already in the book), for the smaller of the two remaining quantities. The incoming order's own limit price is never the trade price.
5. An incoming order keeps trading (possibly across several orders and several price levels) until it is filled or no compatible order remains. Whatever is left **rests in the book** at its limit price, behind the orders already at that price. Partial fills are normal. An order that is completely filled never rests.
6. A resting order can be **cancelled**. A partially filled order can be cancelled too; only its remaining quantity is withdrawn (past trades stay). A cancelled order never trades again.
7. Observers can ask for the best bid and best ask, the total resting quantity at a price level, and how much of a given order is still open.
8. Every trade is recorded in execution order.

## Constraints

- Single-threaded access.
- Handle on the order of 200,000 orders and cancels with prices spread over tens of thousands of levels: no operation may scan the whole book.
- Only trade when prices cross: the best bid is always strictly below the best ask after any operation.

## Examples

(Ids shown assume the first order gets id 1, the next id 2, and so on. Only uniqueness is required of your ids.)

Trade at the resting price, remainder rests:

```
submit(SELL, 100, 5) -> id 1       rests;   bestAsk = 100
submit(BUY, 105, 8)  -> id 2       trades 5 @ 100 with order 1 (the resting price, not 105)
                                   trades() = ["2:1:100:5"]       (format buyId:sellId:price:quantity)
                                   remaining(1) = 0, remaining(2) = 3, bestBid = 105, bestAsk = none
submit(BUY, 99, 1)   -> id 3       rests behind;  volumeAt(BUY, 105) = 3, volumeAt(BUY, 99) = 1
```

Best price first, then sweeping several levels:

```
SELL 1 @ 102 (id 1), SELL 1 @ 101 (id 2), SELL 1 @ 103 (id 3)
BUY 3 @ 102 (id 4)  -> trades "4:2:101:1" (cheapest first), then "4:1:102:1"; 103 is above its limit
                       order 4 keeps 1 and rests at 102: bestBid = 102, bestAsk = 103
```

Earliest first at the same price, and a sell hitting a resting buy:

```
SELL 2 @ 100 (id 1), SELL 2 @ 100 (id 2), BUY 3 @ 100 (id 3)
   -> trades "3:1:100:2", "3:2:100:1"; remaining(2) = 1, volumeAt(SELL, 100) = 1

BUY 5 @ 100 (id 1) rests, then SELL 5 @ 90 (id 2)   -> trade "1:2:100:5" (buy id first, resting price 100)
```

Cancel:

```
BUY 10 @ 99 (id 1), BUY 5 @ 98 (id 2)    cancel(1) -> true; remaining(1) = 0; bestBid = 98
cancel(1) again -> false          cancel(9999) -> false
SELL 10 @ 100 (id 1), BUY 4 @ 100 (id 2) trade; cancel(1) -> true withdraws only the 6 left; trades() still holds the trade
```

## Edge cases

- Invalid submissions return empty and change nothing: side other than exactly `"BUY"`/`"SELL"` (so `"buy"` and `"HOLD"` are invalid), price `<= 0`, quantity `<= 0`.
- `cancel` returns `false` for ids that are unknown, fully filled, or already cancelled.
- `remaining(id)`: `0` for filled or cancelled orders, empty for an id that was never issued, and also empty on a brand-new book.
- `volumeAt` with a side that is not valid, or at a price with no resting orders, is `0`. It adds up only orders of that side at exactly that price, and drops cancelled or filled quantity.
- When the best level is used up, the next-best level becomes best (e.g. bids at 100, 99, 98 and an incoming `SELL 2 @ 99` trades the 100 and 99 bids, leaving best bid 98).
- `trades()` is a snapshot: trades made after you called it do not appear in the list you already got.

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

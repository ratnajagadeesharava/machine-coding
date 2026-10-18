# Interviewer notes — 17 Stock Exchange Order Book

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- A multi-level sweep ending with a remainder resting at the incoming limit price.
- A partially filled resting order keeps its place in the queue.
- Cancelling the head of a price level promotes the next order.
- Trades by an incoming sell execute at the resting bid price, not the sell limit.
- Equal prices cross.

## Likely interviewer questions
- Which structure holds each side so best price, insert and cancel are all cheap? What is each complexity?
- How is time priority kept inside a price level, and how do you cancel from the middle of it in O(1)?
- What happens to price-level bookkeeping when a level becomes empty (stale heap entries, removing from a sorted map)?
- How do you reconcile "order id → location in the book" with matching and cancel?
- How would market orders and amendments change your structure?

## Invariants
- After every operation best bid < best ask (if both exist).
- For every order: quantity = filled + remaining + cancelled.
- `volumeAt(side, price)` = Σ remaining of the resting orders at that level.
- A trade's quantity never exceeds either order's remaining quantity before it.

## Common mistakes
- Trading at the incoming order's price (or the midpoint) instead of the resting price.
- Losing time priority (re-sorting by quantity, LIFO queue, unstable heap ties).
- O(n) scans for best price or for cancel; `List.remove` on a queue of resting orders.
- Leaving a zero-quantity order in the book, or an empty price level that still counts as best.
- Matching the same side or crossing orders left over from partial fills.
- Reporting the sell id first in trades.

## Over-engineering traps
- `Order`, `Trade`, `PriceLevel`, `OrderBook`, `MatchingEngine`, `OrderType` hierarchy, `OrderFactory`, event bus in V1.
- Interfaces for matching strategies when the rule set is one sentence long.

## Scalability discussion
- Sorted map price → FIFO queue (tree map + deque) gives O(log P) insert and O(1) best; cancel needs an id index and lazy deletion or intrusive links.
- Heap of prices with lazy deletion of empty levels; array of levels when prices are bounded integers.

## Trade-offs worth probing
- Lazy cancel (tombstones, skip at match time) vs eager unlink: memory vs complexity.
- Whole-book `trades()` list vs a bounded ring buffer.

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.

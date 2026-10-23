# 90 · Food Delivery Dispatch (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 75 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building the backend of a small food-delivery app on a city grid. Restaurants serve a cuisine and keep a limited stock of every dish. Customers find restaurants near them and place an order; the restaurant accepts and cooks it, and once it is ready a delivery partner has to carry it to the customer. Partners are scarce: each carries one order at a time, the closest free one gets the order, and when everybody is busy ready orders wait in line and are dispatched the moment somebody becomes free. Customers can cancel until the food is on its way, and a cancelled order must give back everything it took (stock and partner).

## Glossary

- **Grid / Manhattan distance**: restaurants and partners sit at integer `(x, y)` points; distance between `(x1,y1)` and `(x2,y2)` is `|x1-x2| + |y1-y2|`. Partners do not move.
- **Order lifecycle**: `PLACED -> ACCEPTED -> READY -> PICKED_UP -> DELIVERED`, or `CANCELLED`.
- **Free partner**: a partner with no assigned order. A partner is busy from assignment until the order is delivered or cancelled.
- **Waiting queue**: READY orders that found no free partner, served first-come-first-served by the moment they became READY.
- **Reserve**: taking dish quantities out of stock when the order is placed; stock goes back on cancel.

## Requirements

1. `addRestaurant(id, cuisine, x, y)`, `addItem(restaurant, item, price, stock)` and `addPartner(id, x, y)` set up the world. Re-adding an item replaces its price and **adds** to its stock.
2. `searchRestaurants(cuisine, x, y, maxDistance)` returns the restaurants of that cuisine within `maxDistance` (Manhattan distance) of the point, nearest first, ties by id.
3. `placeOrder(customer, restaurant, items)` takes a map `item -> quantity` (all quantities >= 1), reserves the stock all-or-nothing and returns an order id. Order total = sum of price x quantity.
4. Orders move through PLACED -> ACCEPTED -> READY -> PICKED_UP -> DELIVERED. `accept`, `markReady`, `pickUp`, `deliver` perform one step each; anything else is a state error.
5. `markReady` also starts the search for a partner: the **free** partner nearest to the restaurant (ties by partner id) gets the order. If nobody is free the order waits in first-come-first-served order and is assigned the moment a partner becomes free. A partner holds at most one order, from assignment until delivery or cancellation.
6. `partnerOf(order)` returns the assigned partner, if any. `pickUp` needs a partner to be assigned already.
7. `deliver` frees the partner, who immediately takes the oldest waiting order, if there is one.
8. `cancel(order, customer)`: only the customer who placed it, only from PLACED, ACCEPTED or READY. Stock is restored, the partner (if any) is freed and re-dispatched, and a cancelled waiting order leaves the queue.
9. `statusOf`, `totalOf` and `stockOf` expose the current state.

## Constraints

- The service is used from many threads. Stock must never go negative or be oversold: 50 threads ordering the last 10 portions yield exactly 10 successes.
- A partner never carries two orders at once, even when many orders become READY at the same instant; there is no lost wake-up (an order that waits is assigned when a partner frees up).
- Partner search must be deterministic: the nearest free partner by Manhattan distance to the restaurant, ties by partner id.
- The hidden tests push 1,600 orders through 3 partners with 16 threads; assignment and the queue must keep up with that.
- Prices and stock are whole numbers; there is no clock in this problem.

## Examples

```
Setup: r1 (pizza, 0,0): margherita 250 x10, cola 50 x100;  r2 (pizza, 5,5);  r3 (sushi, 1,1);  partner p1 at (0,0)

search_restaurants("pizza", 1, 1, 100) -> [r1, r2]   (distances 2 and 8)
search_restaurants("pizza", 5, 4, 100) -> [r2, r1]   (distances 1 and 9)
search_restaurants("pizza", 1, 1, 3)   -> [r1]       search_restaurants("sushi", 0, 0, 1) -> []  (r3 is 2 away)

place_order(ann, r1, {margherita:2, cola:3}) -> o1    total_of(o1) = 2*250 + 3*50 = 650, status PLACED
stock_of(r1, margherita) = 8, stock_of(r1, cola) = 97
place_order(ann, r1, {cola:5, margherita:11}) -> out-of-stock error; stock stays cola 100, margherita 10 (nothing reserved)

Queueing for a partner
accept(o1); mark_ready(o1)  -> partner_of(o1) = p1
accept(o2); mark_ready(o2)  -> partner_of(o2) = none (p1 busy, o2 waits)
pick_up(o2) -> state error (no partner yet)
pick_up(o1); deliver(o1)    -> p1 is free again and immediately takes o2: partner_of(o2) = p1

Nearest free partner, ties by id
partners far(9,9), pb(1,0), pa(0,1); restaurant at (0,0); three orders become READY one after another
-> served by pa, pb, far (pa and pb are both distance 1; "pa" < "pb")

Cancel
o1 READY and holding p1, o2 READY and waiting; cancel(o1, ann) -> o1 CANCELLED, stock restored, p1 goes to o2
o1, o2, o3 READY with one partner, o2 cancelled while waiting -> after o1 is delivered, p1 takes o3 (o2 left the queue)
cancel(o, other customer) -> not allowed; cancel after PICKED_UP, or a second cancel -> state error
```

## Edge cases

- `place_order` with an unknown restaurant or item, an empty item map, or any quantity `< 1` is invalid input; not enough stock for any line is an out-of-stock error and nothing is reserved for the other lines.
- Order total is fixed at placement (later `add_item` price changes do not affect it). Re-adding an item replaces its price and adds to its stock.
- Only the exact next transition is allowed: e.g. `mark_ready` on a PLACED order, `accept` twice, `deliver` before `pick_up`, or `pick_up` on a READY order with no partner yet are state errors. An unknown order id is an error of its own kind (unknown id).
- `partner_of` is empty for orders that are waiting, never got a partner, or were cancelled while waiting.
- 50 threads each ordering one portion of a dish with stock 10: exactly 10 succeed and the stock ends at 0.
- With `n` partners and many orders becoming READY concurrently, no partner ever holds two orders and every waiting order is eventually assigned when a partner frees up.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `FoodDeliveryApi`; `IllegalArgumentException` for an unknown restaurant, item or order id and for an empty order or quantity < 1; `IllegalStateException` for out-of-stock and for any illegal transition (including `pickUp` without a partner and `cancel` after PICKED_UP or a second `cancel`); `SecurityException` for cancel by another customer. Rust: `FoodDelivery` trait with `FoodError::{UnknownId, InvalidInput, OutOfStock, InvalidState, NotAllowed}`; `add_item`, `partner_of`, `stock_of` return `Food<..>`.
- Status strings are exactly `PLACED`, `ACCEPTED`, `READY`, `PICKED_UP`, `DELIVERED`, `CANCELLED`.
- Out-of-stock for any line refuses the whole order and reserves nothing; the stock after a refused order is unchanged.
- A cancel restores exactly the reserved quantities; the order total is fixed when it is placed.
- Tie-break for partners: equal distance -> smaller partner id. With partners far(9,9), pb(1,0), pa(0,1) and a restaurant at (0,0) three READY orders are served by `pa`, `pb`, `far` in that order.
- The waiting queue is FIFO by the moment `markReady` was called; cancelling a waiting order removes it, and the freed partner goes to the next one.
- Freed partners are re-dispatched from three places: `deliver`, `cancel` of an order that holds a partner, and (implicitly) a later `markReady` that finds a free partner.
- `partnerOf` is empty both for orders waiting for a partner and for orders that never got one (and for cancelled waiting orders).

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! Everything lives on an integer grid; distance is Manhattan distance |x1-x2| + |y1-y2|.
//!
//! Order states: PLACED -> ACCEPTED -> READY -> PICKED_UP -> DELIVERED, or CANCELLED (from PLACED, ACCEPTED or READY).

#[derive(Debug, PartialEq, Eq)]
pub enum FoodError {
    /// unknown restaurant, item or order id
    UnknownId,
    /// empty order, quantity 0
    InvalidInput,
    OutOfStock,
    /// transition not allowed from the current state (or pickUp without a partner)
    InvalidState,
    /// cancel by somebody else than the customer
    NotAllowed,
}

pub type Food<T> = Result<T, FoodError>;

pub trait FoodDelivery: Send + Sync {
    fn add_restaurant(&self, id: &str, cuisine: &str, x: i32, y: i32);

    /// A menu item with `stock` portions available. Re-adding an item replaces its price and adds to the stock.
    fn add_item(&self, restaurant_id: &str, item: &str, price: u64, stock: u32) -> Food<()>;

    fn add_partner(&self, partner_id: &str, x: i32, y: i32);

    /// Restaurants of the cuisine within `max_distance` of (x, y), nearest first, ties by id.
    fn search_restaurants(&self, cuisine: &str, x: i32, y: i32, max_distance: u32) -> Vec<String>;

    /// Places an order for `item -> quantity` (all quantities >= 1) and reserves the stock, all-or-nothing.
    fn place_order(
        &self,
        customer: &str,
        restaurant_id: &str,
        items: &[(&str, u32)],
    ) -> Food<String>;

    fn total_of(&self, order_id: &str) -> Food<u64>;

    fn status_of(&self, order_id: &str) -> Food<String>;

    /// PLACED -> ACCEPTED.
    fn accept(&self, order_id: &str) -> Food<()>;

    /// ACCEPTED -> READY and the order looks for a delivery partner: the FREE partner nearest to the restaurant
    /// (ties by partner id) gets it. If nobody is free the order waits (first come, first served) and is assigned
    /// the moment a partner becomes free. A partner has at most one order at a time, from assignment until delivery
    /// or cancellation.
    fn mark_ready(&self, order_id: &str) -> Food<()>;

    /// The partner assigned to the order, if any.
    fn partner_of(&self, order_id: &str) -> Food<Option<String>>;

    /// READY (with a partner assigned) -> PICKED_UP.
    fn pick_up(&self, order_id: &str) -> Food<()>;

    /// PICKED_UP -> DELIVERED; the partner becomes free (and takes the oldest waiting order, if any).
    fn deliver(&self, order_id: &str) -> Food<()>;

    /// Cancels the order, restores its stock and frees its partner. Only the customer who placed it.
    fn cancel(&self, order_id: &str, customer: &str) -> Food<()>;

    /// Portions of the item still available for new orders.
    fn stock_of(&self, restaurant_id: &str, item: &str) -> Food<u32>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create() -> Option<Arc<dyn FoodDelivery>> { ... }
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Time-dependent behaviour uses an injectable fake clock, never real sleeping. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Restaurant search by cuisine and distance, ordering by distance then id, different reference points, empty results.
- Order placement: total, status PLACED, stock reservation; all-or-nothing when one line is short; invalid orders (unknown restaurant, unknown item, quantity 0, empty order, unknown order id).
- Happy path through every state; illegal transitions in each direction (ready before accept, pickUp before ready, double accept, deliver too early, cancel after pickup, double deliver).
- Nearest free partner with id tie-break across three consecutive orders.
- A single partner and three READY orders: the others wait, `pickUp` without a partner is a state error, they get the partner in FIFO order after each delivery.
- Cancel restores stock, frees the partner so the waiting order gets it, a second cancel fails; cancelling a waiting order removes it from the queue; only the customer can cancel.
- Concurrency: 50 threads for 10 portions (exactly 10 win); 12 threads x 25 complete order flows with 4 partners (no partner ever holds two orders; all 300 delivered); 6 threads making orders READY with 2 partners, repeated 100x (exactly two get distinct partners).
- More edge cases (500 concurrent cancels restoring stock, bursts of READY orders with few partners) are in `interviewer/`.

## Running

```bash
cargo test -p p90_food_delivery
STRICT=1 cargo test -p p90_food_delivery   # fail instead of skip while unimplemented
cargo test -p p90_food_delivery --features hidden   # interviewer scenarios
cargo play run 90              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`FakeClock`, `sut!`): `rust/support`.

# 65 · Observer (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — `PriceObserver` and `FeedError` provided; write `PriceFeed`. |

## Problem

A stock-ticker service publishes prices. Dashboards, alert engines and loggers all want to react to price changes,
but the ticker must not depend on any of them — and one broken listener must never stop the others.

**What you are building:** a `PriceFeed` that other components can sign up with. A component can ask to hear about
every stock symbol or only about one (say `"ACME"`). Whenever a new price is published, every interested listener
is called, in the order they signed up. A listener can later cancel its sign-up, and a listener that fails must
not prevent the rest from being told.

### Provided types

- `PriceObserver::on_price(&self, symbol: &str, price_cents: u64) -> Result<(), String>` — the listener callback; returning `Err` means the observer failed. Prices are whole cents (`1250` = $12.50).
- `FeedError::{InvalidSymbol, InvalidPrice}` — `InvalidSymbol` for a blank symbol, `InvalidPrice` for a price of 0.
- `SubscriptionId = u64`.

### Glossary

- **Observer / subscriber**: a listener that wants to be told about prices.
- **Subscription**: one sign-up, identified by a `SubscriptionId`. The same observer (`Rc`) can hold several subscriptions.
- **Active**: subscribed and not unsubscribed.
- **Blank symbol**: empty or only whitespace.

## Requirements

`PriceFeed` (created with `PriceFeed::new()`; every method takes `&self`):

1. `subscribe(observer)` listens to every symbol and returns a `SubscriptionId`; `subscribe_to(symbol, observer)` listens to one symbol (exact, case-sensitive match; blank symbol → `Err(FeedError::InvalidSymbol)`). Ids are unique across all subscriptions.
2. `unsubscribe(id)` stops notifications; it returns `true` the first time and `false` afterwards (and for an unknown id). `subscriber_count()` counts only active subscriptions (every symbol included).
3. `publish(symbol, price_cents)` (blank symbol → `Err(InvalidSymbol)`, price 0 → `Err(InvalidPrice)`, nobody notified) calls `on_price(symbol, price_cents)` on each interested observer **in subscription order**, with the symbol and price exactly as published. Interested means subscribed to all symbols, or to exactly this symbol. Subscription order is one order across both kinds. Publishing when nobody is subscribed returns `Ok(0)`.
4. **Snapshot semantics:** an observer that subscribes *during* a publish does not receive that price (it gets later ones).
5. An observer unsubscribed before its turn — even by an earlier observer during the same publish — is **skipped**.
6. An observer that returns `Err` does not stop the others; `publish` returns `Ok(number_that_failed)`. A failing observer stays subscribed (it is still counted by `subscriber_count()` and is notified on later publishes).
7. Subscribing the same `Rc` twice means it is notified twice (once per subscription, in subscription order); unsubscribing one subscription leaves the other.

## Examples

```rust
feed.subscribe_to("ACME", acme_only)?;   // subscription 1 (logs "acme-only ACME=100")
feed.subscribe(all);                      // subscription 2 (logs "all ACME=100")

feed.publish("ACME", 100)  // Ok(0); log: "acme-only ACME=100", "all ACME=100"
feed.publish("INIT", 200)  // Ok(0); log: "all INIT=200"
```

Unsubscribe and count:

```rust
let a = feed.subscribe(obs_a);
feed.subscribe(obs_b);
feed.unsubscribe(a)   // true
feed.unsubscribe(a)   // false
feed.subscriber_count()   // 1
feed.publish("X", 2)  // only obs_b is notified
```

Failures (four subscriptions: logs "ok-1", returns Err, returns Err, logs "ok-2"):

```rust
feed.publish("X", 5)  // Ok(2); log: "ok-1 X=5", "ok-2 X=5"; subscriber_count() is still 4
```

Skipped when unsubscribed mid-publish (first observer unsubscribes the second; a third just logs):

```rust
feed.publish("X", 1)  // log: "first 1", "third X=1"   (second never called)
```

Subscribe during publish (the single observer subscribes a new "late" observer each time it is called):

```rust
feed.publish("X", 1)  // early observer called; "late" not called for price 1
feed.publish("X", 2)  // early observer called, then the "late" one added during publish 1 is called with 2
```

## Edge cases

- `publish(" ", 1)` → `Err(InvalidSymbol)`; `publish("X", 0)` → `Err(InvalidPrice)`; neither notifies anyone, even if observers exist.
- `subscribe_to(" ", obs)` → `Err(InvalidSymbol)`.
- Same `Rc` subscribed twice, one subscription removed: it is notified once per publish afterwards.

## Running

```bash
cargo test -p p65_observer     # from rust/ — tests FAIL until you implement PriceFeed
cargo play run 65
```

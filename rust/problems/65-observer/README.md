# 65 · Observer (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — `PriceObserver` and `FeedError` provided; write `PriceFeed`. |

## Problem

A stock-ticker service publishes prices. Dashboards, alert engines and loggers all want to react to price changes,
but the ticker must not depend on any of them — and one broken listener must never stop the others.

## Requirements

`PriceFeed`:

1. `subscribe(observer)` listens to every symbol; `subscribe_to(symbol, observer)` to one symbol (exact match; blank symbol → `Err(FeedError::InvalidSymbol)`). Both return unique `SubscriptionId`s.
2. `unsubscribe(id)` stops notifications; it returns `true` the first time and `false` afterwards. `subscriber_count()` counts only active subscriptions.
3. `publish(symbol, price_cents)` (blank symbol → `Err(InvalidSymbol)`, price 0 → `Err(InvalidPrice)`, nobody notified) calls `on_price` on each interested observer **in subscription order**.
4. **Snapshot semantics:** an observer that subscribes *during* a publish does not receive that price.
5. An observer unsubscribed before its turn — even by an earlier observer during the same publish — is **skipped**.
6. An observer that returns `Err` does not stop the others; `publish` returns `Ok(number_that_failed)`.
7. Subscribing the same `Rc` twice means it is notified twice.

## Running

```bash
cargo test -p p65_observer     # from rust/ — tests FAIL until you implement PriceFeed
cargo play run 65
```

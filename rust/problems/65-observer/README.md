# 65 · Observer (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — the `PriceObserver` trait and `FeedError` are provided; `PriceFeed` is a stub. |

> **Pattern:** *Observer* — a subject keeps a list of observers and notifies them when its state changes, without
> knowing what the observers are or do. Subscribers come and go at runtime and are loosely coupled to the subject.
>
> **Rust adaptation:** observers are `Rc<dyn PriceObserver>` and every `PriceFeed` method takes `&self`, so an
> observer that holds an `Rc<PriceFeed>` can subscribe or unsubscribe while a notification is running (the feed
> keeps its list in a `RefCell`; copy the list before notifying so you never hold a borrow while calling out).
> Failure is a returned `Err`, not an exception. Subscriptions are identified by a `SubscriptionId`.

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

# 65 · Observer (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — `PriceObserver` and `Subscription` provided; write `PriceFeed`. |

## Problem

A stock-ticker service publishes prices. Dashboards, alert engines and loggers all want to react to price changes,
but the ticker must not depend on any of them — and one broken listener must never stop the others.

## Requirements

`PriceFeed`:

1. `subscribe(observer)` listens to every symbol; `subscribe(symbol, observer)` to one symbol (exact match). Both return a `Subscription`. `null` observer / blank symbol → `IllegalArgumentException`.
2. `Subscription.cancel()` stops notifications and may be called repeatedly; `isActive()` reflects it. `subscriberCount()` counts only active subscriptions.
3. `publish(symbol, priceCents)` (blank symbol or price `<= 0` → `IllegalArgumentException`, nobody notified) calls `onPrice(symbol, price)` on each interested observer **in subscription order**.
4. **Snapshot semantics:** an observer that subscribes *during* a publish does not receive that price (it gets later ones).
5. An observer that is cancelled before its turn — even by an earlier observer during the same publish — is **skipped**.
6. An observer that throws does not stop the others; `publish` returns how many threw.
7. Subscribing the same observer object twice means it is notified twice; cancelling one subscription leaves the other.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement PriceFeed
```

# 65 · Observer (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 40 min | Level 2 — `PriceObserver` and `Subscription` provided; write `PriceFeed`. |

## Problem

A stock-ticker service publishes prices. Dashboards, alert engines and loggers all want to react to price changes,
but the ticker must not depend on any of them — and one broken listener must never stop the others.

**What you are building:** a `PriceFeed` that other components can sign up with. A component can ask to hear about
every stock symbol or only about one (say `"ACME"`). Whenever a new price is published, every interested listener
is called, in the order they signed up. A listener can later cancel its sign-up, and a listener that crashes must
not prevent the rest from being told.

### Provided types

- `PriceObserver.onPrice(String symbol, long priceCents)` — the listener callback. Prices are whole cents (`1250` = $12.50).
- `Subscription` — handle returned by `subscribe`: `cancel()` (safe to call any number of times) and `isActive()`.

### Glossary

- **Observer / subscriber**: a listener that wants to be told about prices.
- **Subscription**: one sign-up. The same observer object can hold several subscriptions.
- **Active**: subscribed and not cancelled.
- **Blank symbol**: `null`, empty, or only whitespace.

## Requirements

`PriceFeed` (has a no-argument constructor):

1. `subscribe(observer)` listens to every symbol; `subscribe(symbol, observer)` to one symbol (exact, case-sensitive match). Both return a `Subscription`. `null` observer / blank symbol → `IllegalArgumentException`.
2. `Subscription.cancel()` stops notifications and may be called repeatedly; `isActive()` is `true` until cancelled, then `false`. `subscriberCount()` counts only active subscriptions (it includes every subscription, whichever symbol it listens to).
3. `publish(symbol, priceCents)` (blank symbol or price `<= 0` → `IllegalArgumentException`, nobody notified) calls `onPrice(symbol, price)` on each interested observer **in subscription order**, with the symbol and price exactly as published. Interested means subscribed to all symbols, or to exactly this symbol. Subscription order is one order across both kinds. Publishing when nobody is subscribed is fine and returns `0`.
4. **Snapshot semantics:** an observer that subscribes *during* a publish does not receive that price (it gets later ones).
5. An observer that is cancelled before its turn — even by an earlier observer during the same publish — is **skipped**.
6. An observer that throws (any `RuntimeException`) does not stop the others; `publish` returns how many threw. A throwing observer stays subscribed (it is still counted by `subscriberCount()` and is notified on later publishes).
7. Subscribing the same observer object twice means it is notified twice (once per subscription, in subscription order); cancelling one subscription leaves the other.

## Examples

```java
feed.subscribe("ACME", (s, p) -> log("acme-only " + p));   // subscription 1
feed.subscribe((s, p) -> log("all " + s));                 // subscription 2

feed.publish("ACME", 100);   // log: "acme-only 100", "all ACME"   -> returns 0
feed.publish("INIT", 200);   // log: "all INIT"                    -> returns 0
```

Cancel and count:

```java
Subscription a = feed.subscribe(obsA);
feed.subscribe(obsB);
a.cancel(); a.cancel();      // second call is harmless
a.isActive();                // false
feed.subscriberCount();      // 1
feed.publish("X", 2);        // only obsB is notified
```

Failures (four subscriptions: logs "ok-1", throws, throws, logs "ok-2"):

```java
feed.publish("X", 5);        // returns 2; log: "ok-1", "ok-2"; subscriberCount() is still 4
```

Skipped when cancelled mid-publish (first observer cancels the second; a third just logs):

```java
feed.publish("X", 1);        // log: "first 1", "third 1"   (second never called)
```

Subscribe during publish (the single observer subscribes a new "late" observer each time it is called):

```java
feed.publish("X", 1);        // early observer called; "late" not called for price 1
feed.publish("X", 2);        // early observer called, then the "late" one added during publish 1 is called with 2
```

## Edge cases

- `publish(" ", 1)` and `publish("X", 0)` throw `IllegalArgumentException` and notify no one, even if observers exist.
- `subscribe(" ", obs)` and `subscribe("X", null)` throw `IllegalArgumentException`; `subscribe(null)` too.
- Same observer subscribed twice, one subscription cancelled: it is notified once per publish afterwards.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement PriceFeed
```

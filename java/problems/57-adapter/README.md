# 57 · Adapter (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — the target (`Notifier`, `Delivery`) and the legacy adaptee (`LegacySmsGateway`) are provided; the adapter and a small client are stubs. |

> **Pattern:** *Adapter* — convert the interface of an existing class into the interface that clients expect, so
> classes with incompatible interfaces can work together **without changing either**.

## Problem

The application sends alerts through a clean `Notifier` interface. The company has just bought a vendor SMS gateway
whose API is ancient: it takes an XML string and returns an integer code. You may not change the vendor code or the
application code. Write the adapter.

## Requirements

1. `LegacySmsAdapter(LegacySmsGateway)` implements `Notifier` by **wrapping** the gateway (object adapter): its only superclass is `Object` and its only interface is `Notifier`.
2. `send(phone, message)`:
   - blank/`null` phone → `Delivery(false, "invalid number")`; then blank message → `Delivery(false, "empty message")`; in both cases the legacy gateway is **not** called;
   - otherwise call `submitXml` **exactly once** with `<sms><to>PHONE</to><body>MESSAGE</body></sms>`, where phone and message have `&` `<` `>` `"` `'` escaped as `&amp;` `&lt;` `&gt;` `&quot;` `&apos;` (escape `&` first);
   - map the code: `0` → `Delivery(true, "")`, `1` → `"invalid number"`, `2` → `"message too long"`, `3` → `"unauthorized"`, any other `N` → `"unknown error (code N)"`.
3. `AlertService(Notifier).alertAll(phones, message)` sends to each phone in order and returns `"<phone>: <error>"` for each failure (empty list if none, or if `phones` is `null`/empty). It knows nothing about XML or codes.

## Constraints (what the tests catch)

- The legacy gateway is a recording fake. XML must match exactly, including escaping.
- `AlertService` is also tested with a plain lambda `Notifier`.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

# 57 · Adapter (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — the target trait (`Notifier`) and the legacy adaptee trait (`LegacySmsGateway`) are provided; the adapter struct and `alert_all` are stubs. |

> **Pattern:** *Adapter* — convert the interface of an existing type into the interface that clients expect, so types
> with incompatible interfaces can work together **without changing either**.

## Problem

The application sends alerts through a clean `Notifier` trait. The company has just bought a vendor SMS gateway whose
API is ancient: it takes an XML string and returns an integer code. You may not change the vendor code or the
application code. Write the adapter.

## Requirements

1. `LegacySmsAdapter::new(Box<dyn LegacySmsGateway>)` implements `Notifier` by **wrapping** the gateway.
2. `send(phone, message)`:
   - blank phone → `Err("invalid number")`; then blank message → `Err("empty message")`; in both cases the legacy gateway is **not** called;
   - otherwise call `submit_xml` **exactly once** with `<sms><to>PHONE</to><body>MESSAGE</body></sms>`, where phone and message have `&` `<` `>` `"` `'` escaped as `&amp;` `&lt;` `&gt;` `&quot;` `&apos;` (escape `&` first);
   - map the code: `0` → `Ok(())`, `1` → `Err("invalid number")`, `2` → `Err("message too long")`, `3` → `Err("unauthorized")`, any other `N` → `Err("unknown error (code N)")`.
3. `alert_all(&dyn Notifier, phones, message)` sends to each phone in order and returns `"<phone>: <error>"` for each failure (empty when none). It knows nothing about XML or codes.

## Constraints (what the tests catch)

- The legacy gateway is a recording fake. XML must match exactly, including escaping.
- `alert_all` is also tested with a plain `Notifier` implementor.

## Running

```bash
cargo test -p p57_adapter     # from rust/ — tests FAIL until you implement the stubs
cargo play run 57
```

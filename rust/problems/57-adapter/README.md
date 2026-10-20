# 57 · Adapter (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — the target trait (`Notifier`) and the legacy adaptee trait (`LegacySmsGateway`) are provided; `LegacySmsAdapter` and `alert_all` are stubs. |

## Problem

You are building the glue between a modern application and an old vendor system. The application sends alerts
(for example "Server down") to phone numbers through a clean `Notifier` trait: `send(phone, message)` returns
`Result<(), String>` where `Err` carries the reason. The company has just bought a vendor SMS gateway whose API is
ancient: it accepts a single XML string and answers with an integer status code. You may not change the vendor code or
the application code, so you write `LegacySmsAdapter`, which lets the application use the gateway as if it were a
`Notifier`, and `alert_all`, which sends one message to many phones and reports which ones failed.

## Terms

- **Target**: the trait the application wants (`Notifier`). **Adaptee**: the vendor trait you must reuse (`LegacySmsGateway`).
- **Blank**: empty, or only whitespace.
- **Escaping**: replacing characters that are special in XML with their entity text, so the message cannot break the XML.

## Requirements

1. `LegacySmsAdapter::new(Box<dyn LegacySmsGateway>)` implements `Notifier`, backed by the given gateway.
2. `send(phone, message)` returns `Result<(), String>`, and performs these steps in order:
   - blank phone → `Err("invalid number")`; otherwise blank message → `Err("empty message")`. In both cases the legacy gateway is **not** called (so a blank phone together with a blank message gives `"invalid number"`);
   - otherwise call `submit_xml` exactly once with `<sms><to>PHONE</to><body>MESSAGE</body></sms>`, where phone and message have `&` `<` `>` `"` `'` escaped as `&amp;` `&lt;` `&gt;` `&quot;` `&apos;` (escape `&` first so existing entities are not double-processed incorrectly);
   - map the returned code: `0` → `Ok(())`, `1` → `Err("invalid number")`, `2` → `Err("message too long")`, `3` → `Err("unauthorized")`, any other `N` → `Err("unknown error (code N)")`.
3. `alert_all(&dyn Notifier, phones, message)` sends the message to each phone, in slice order, through the notifier it was given (any `Notifier`, not only the adapter). It returns one string `"<phone>: <error>"` for each failed send, in the same order; successes add nothing. The result is empty when nothing failed or `phones` is empty.

## Examples

| Call | Gateway receives | Result |
|---|---|---|
| `send("+15551234", "Hello there")`, gateway returns 0 | `<sms><to>+15551234</to><body>Hello there</body></sms>` | `Ok(())` |
| `send("1", "A&B <c> \"d\" 'e'")`, gateway returns 0 | `<sms><to>1</to><body>A&amp;B &lt;c&gt; &quot;d&quot; &apos;e&apos;</body></sms>` | `Ok(())` |
| `send("1", "m")`, gateway returns 42 | one call | `Err("unknown error (code 42)")` |
| `send(" ", "m")` / `send("1", "")` / `send("", "")` | no call | `Err("invalid number")` / `Err("empty message")` / `Err("invalid number")` |

`alert_all(&adapter, &["111", "BAD", "222"], "Server down")` where the gateway rejects only number `BAD` with code 1 → `["BAD: invalid number"]`, and the gateway was called 3 times.
With a custom notifier that fails numbers starting with `0` with `"blocked"`: `alert_all(&n, &["0100", "200", "0300"], "hi")` → `["0100: blocked", "0300: blocked"]`.

## Edge cases

- A message made only of special characters is still escaped, not rejected.
- Each `send` makes at most one gateway call; repeated sends are independent.

## Running

```bash
cargo test -p p57_adapter     # from rust/ — tests FAIL until you implement the stubs
cargo play run 57
```

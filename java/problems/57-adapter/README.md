# 57 · Adapter (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 35 min | Level 2 — the target (`Notifier`, `Delivery`) and the legacy adaptee (`LegacySmsGateway`) are provided; `LegacySmsAdapter` and `AlertService` are stubs. |

## Problem

You are building the glue between a modern application and an old vendor system. The application sends alerts
(for example "Server down") to phone numbers through a clean `Notifier` interface: `send(phone, message)` returns a
`Delivery(ok, error)`. The company has just bought a vendor SMS gateway whose API is ancient: it accepts a single XML
string and answers with an integer status code. You may not change the vendor code or the application code, so you write
two classes: `LegacySmsAdapter`, which lets the application use the gateway as if it were a `Notifier`, and
`AlertService`, which sends one message to many phones and reports which ones failed.

## Terms

- **Target**: the interface the application wants (`Notifier`). **Adaptee**: the vendor interface you must reuse (`LegacySmsGateway`).
- **Blank**: `null`, empty, or only whitespace.
- **Escaping**: replacing characters that are special in XML with their entity text, so the message cannot break the XML.

## Requirements

1. `LegacySmsAdapter(LegacySmsGateway)` implements `Notifier`, backed by the given gateway.
2. `send(phone, message)` returns a `Delivery`, and performs these steps in order:
   - blank/`null` phone → `Delivery(false, "invalid number")`; otherwise blank message → `Delivery(false, "empty message")`. In both cases the legacy gateway is **not** called (so a blank phone together with a blank message gives `"invalid number"`);
   - otherwise call `submitXml` exactly once with `<sms><to>PHONE</to><body>MESSAGE</body></sms>`, where phone and message have `&` `<` `>` `"` `'` escaped as `&amp;` `&lt;` `&gt;` `&quot;` `&apos;` (escape `&` first so existing entities are not double-processed incorrectly);
   - map the returned code: `0` → `Delivery(true, "")`, `1` → `Delivery(false, "invalid number")`, `2` → `"message too long"`, `3` → `"unauthorized"`, any other `N` → `"unknown error (code N)"` (always `ok=false`, `error` holds the text).
3. `AlertService(Notifier).alertAll(phones, message)` sends the message to each phone, in list order, through the notifier it was given (any `Notifier`, not only the adapter). It returns one string `"<phone>: <error>"` for each failed delivery, in the same order; successes add nothing. The result is an empty list if nothing failed, or if `phones` is `null` or empty.

## Examples

| Call | Gateway receives | Result |
|---|---|---|
| `send("+15551234", "Hello there")`, gateway returns 0 | `<sms><to>+15551234</to><body>Hello there</body></sms>` | `Delivery(true, "")` |
| `send("1", "A&B <c> \"d\" 'e'")`, gateway returns 0 | `<sms><to>1</to><body>A&amp;B &lt;c&gt; &quot;d&quot; &apos;e&apos;</body></sms>` | `Delivery(true, "")` |
| `send("1", "m")`, gateway returns 42 | one call | `Delivery(false, "unknown error (code 42)")` |
| `send(" ", "m")` / `send(null, "m")` / `send("1", "")` / `send("", "")` | no call | `"invalid number"` / `"invalid number"` / `"empty message"` / `"invalid number"` |

`alertAll(List.of("111", "BAD", "222"), "Server down")` where the gateway rejects only number `BAD` with code 1 → `["BAD: invalid number"]`, and the gateway was called 3 times.
With a custom notifier that fails numbers starting with `0` using error `"blocked"`: `alertAll(List.of("0100", "200", "0300"), "hi")` → `["0100: blocked", "0300: blocked"]`.

## Edge cases

- Message made only of special characters is still escaped, not rejected.
- `alertAll` with `null` phones returns an empty list (no exception).
- Each `send` makes at most one gateway call; repeated sends are independent.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

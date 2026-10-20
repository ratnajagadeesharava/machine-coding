# 62 · Bridge (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 50 min | Level 2 — the `Channel` interface is provided; three channels, the abstract `Alert` and three alert types are stubs. |

## Problem

An ops platform sends three kinds of alerts over three channels, and the combination changes per team and sometimes
at runtime ("page them if email bounced"). Alerts must not know how channels format text, and channels must not
know what an alert is.

## Requirements

1. `Alert` (abstract) holds a `Channel` (the bridge): constructor and `setChannel` reject `null` with `IllegalArgumentException`; `channel()` returns the current one; `abstract void send(recipient, message)`.
2. `InfoAlert` delivers immediately with title `"Info"`, message unchanged. `UrgentAlert` delivers immediately with title `"URGENT"` and the message in upper case.
3. `DigestAlert.send` only **queues** (per recipient). `pending(recipient)` counts the queue. `flush(recipient)` delivers **one** message — title `"Digest (<n>)"`, body = queued messages joined with `"; "` — empties that recipient's queue and returns `true`; with nothing queued it returns `false` and delivers nothing.
4. Channels take a `Consumer<String> wire` (null → `IllegalArgumentException`) and write one line per delivery:
   - `EmailChannel`: `EMAIL to:<recipient> subject:<title> | <body>`
   - `SlackChannel`: `SLACK @<recipient> *<title>* <body>`
   - `SmsChannel`: `SMS <recipient>: <text>` with `text = title + ": " + body`, cut to **40 characters** — a longer text becomes its first 37 characters followed by `...`.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement the classes
```

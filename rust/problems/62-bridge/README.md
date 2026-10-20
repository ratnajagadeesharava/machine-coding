# 62 · Bridge (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 50 min | Level 2 — the `Channel` trait is provided in `src/contract.rs`; three channels and three alert types (plus the `Alert` trait) are stubs. |

## Problem

An ops platform sends three kinds of alerts over three channels, and the combination changes per team and sometimes
at runtime. Alerts must not know how channels format text, and channels must not know what an alert is.

## Requirements

1. `InfoAlert` delivers immediately with title `"Info"`, message unchanged. `UrgentAlert` delivers immediately with title `"URGENT"` and the message in upper case. Both can `set_channel` at runtime.
2. `DigestAlert::send` only **queues** (per recipient). `pending(recipient)` counts the queue. `flush(recipient)` delivers **one** message — title `"Digest (<n>)"`, body = queued messages joined with `"; "` — empties that recipient's queue and returns `true`; with nothing queued it returns `false` and delivers nothing.
3. Channels take a `Box<dyn Fn(&str)>` wire and write one line per delivery:
   - `EmailChannel`: `EMAIL to:<recipient> subject:<title> | <body>`
   - `SlackChannel`: `SLACK @<recipient> *<title>* <body>`
   - `SmsChannel`: `SMS <recipient>: <text>` with `text = title + ": " + body`, cut to **40 characters** — a longer text becomes its first 37 characters followed by `...`.

## Running

```bash
cargo test -p p62_bridge     # from rust/ — tests FAIL until you implement the stubs
cargo play run 62
```

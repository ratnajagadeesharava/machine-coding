# 62 · Bridge (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 50 min | Level 2 — the `Channel` trait is provided in `src/contract.rs`; three channels and three alert types (plus the `Alert` trait) are stubs. |

## Problem

An ops platform sends three kinds of alerts (info, urgent, digest) over three channels (email, Slack, SMS), and the
combination changes per team and sometimes at runtime. An alert decides **what** is said (title and body); a channel
decides **how** it is written out. Alerts must not know how channels format text, and channels must not know what an alert
is. The provided `Channel` trait is `deliver(&self, recipient, title, body)`; the `Alert` trait (in `src/lib.rs`) has
`send(&mut self, recipient, message)` and `set_channel(&mut self, channel)`.

## Terms

- **Wire**: a `Box<dyn Fn(&str)>` that a channel calls with its finished text line; tests collect these lines.
- **Digest**: several messages for one recipient combined into a single delivery.
- **Flush**: send the digest now.

## Requirements

1. `InfoAlert` delivers immediately with title `"Info"`, message unchanged. `UrgentAlert` delivers immediately with title `"URGENT"` and the message in upper case. Both (and `DigestAlert`) can `set_channel` at runtime; later sends use the new channel.
2. `DigestAlert::send` only **queues** the message for that recipient (nothing is delivered). `pending(recipient)` is the number of queued messages for them. `flush(recipient)` delivers **one** message — title `"Digest (<n>)"` with `n` the number queued, body = queued messages joined with `"; "` in the order queued — empties that recipient's queue and returns `true`; with nothing queued it returns `false` and delivers nothing. Other recipients' queues are untouched.
3. Channels take a `Box<dyn Fn(&str)>` wire and write one line per delivery:
   - `EmailChannel`: `EMAIL to:<recipient> subject:<title> | <body>`
   - `SlackChannel`: `SLACK @<recipient> *<title>* <body>`
   - `SmsChannel`: `SMS <recipient>: <text>` with `text = title + ": " + body`; if `text` is longer than **40 characters** it becomes its first 37 characters followed by `...` (so exactly 40); 40 characters or fewer is left as is.
4. Alerts work with any `Channel` implementation, including ones supplied by a caller.

## Examples

| Call | Line(s) written / delivered |
|---|---|
| `InfoAlert::new(email).send("ann", "ping")` | `EMAIL to:ann subject:Info \| ping` |
| `UrgentAlert::new(slack).send("ann", "ping")` | `SLACK @ann *URGENT* PING` |
| `InfoAlert::new(slack).send("ann", "ping")`, then `set_channel(email)`, `send("ann", "second")` | `SLACK @ann *Info* ping`, then `EMAIL to:ann subject:Info \| second` |
| digest: `send(ann,"one")`, `send(bob,"other")`, `send(ann,"two")`, `send(ann,"three")` | nothing yet; `pending("ann")` = 3 |
| then `flush("ann")` → `true` | one delivery to ann: title `Digest (3)`, body `one; two; three`; `pending("ann")` = 0, `pending("bob")` = 1 |
| `flush("ann")` again | `false`, nothing delivered |
| `SmsChannel::new(w).deliver("+1555", "Report", "All good")` | `SMS +1555: Report: All good` |
| SMS, title `T`, body of 37 `x` / of 38 `x` | text `T: ` + 37 x (40 chars, unchanged) / `T: ` + 34 x + `...` (40 chars) |

## Running

```bash
cargo test -p p62_bridge     # from rust/ — tests FAIL until you implement the stubs
cargo play run 62
```

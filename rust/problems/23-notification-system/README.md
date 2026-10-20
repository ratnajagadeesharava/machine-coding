# 23 · Notification System (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 90 min | Level 2 — minimal external API. The public interface is part of the problem; everything behind it is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building the core of the notification service behind an app, the part that sends "your login code is 42" or "your order shipped". Application code asks it to notify one user about something, giving a message template and the channels it would like to use (for example `email`, `sms`, `push`) in order of preference. The service fills in the template, honours what the user has opted out of, tries the channels one by one (retrying a flaky channel a few times) and tells the caller exactly what happened. The actual delivery to the outside world is performed by a **delivery gateway** that is given to you; it is the only outside dependency, and it is slow, so it must not be called needlessly.

**Terms**

- **Channel**: a way of reaching a user, named by a free-form, case-sensitive string such as `email`.
- **Category**: the kind of message, such as `marketing` or `security`. Opt-outs are per channel *and* category.
- **Eligible channel**: a requested channel for which the user has an address and which is not switched off for the message's category.
- **Placeholder**: `{name}` in a template body, where `name` is one or more letters, digits or underscores.
- **Attempt**: one call to the gateway. Accepted means it returned `true`, refused means `false`.

## Requirements

1. Users are registered with an address per channel (for example `email` -> `a@x.com`, `sms` -> `555-0100`). Channel names are free-form, case-sensitive strings. Registering a user again replaces their addresses but keeps their preferences.
2. A user can opt a channel in or out **per category** (for example `marketing`, `security`). Everything is enabled until someone says otherwise, and a preference can be switched back on.
3. A message template has an id and a body containing `{name}` placeholders, where a name is one or more letters, digits or underscores. Defining a template again replaces it. Text in braces that is not such a placeholder is ordinary text.
4. `send` renders the template with the given parameters, then goes through the requested channels **in the order given**:
   - a channel is skipped when the user has no address for it, or has it switched off for the message's category;
   - an eligible channel gets **up to 3 delivery attempts**; the first attempt the gateway accepts ends the whole send as delivered;
   - when all 3 attempts on a channel are refused, the next eligible channel is tried;
   - a channel that appears more than once in the request is used once, at its first position.
5. Every delivery attempt hands the gateway the channel, the user's address for it and the rendered body. The body is rendered once per send.
6. Rendering replaces every occurrence of each placeholder with its parameter value; parameters that the template does not mention are ignored. Replacement happens in a single pass: text coming from a parameter value is never searched for placeholders again.
7. Nothing is sent when the request cannot be fulfilled or when the user has chosen not to receive it. The gateway must not be called at all in those cases.

## Constraints

- One caller at a time; no threading requirements.
- The gateway is slow and external: never call it more often than the rules above require.
- Tests supply a gateway that records calls and can be scripted to refuse attempts.

## Examples

User `u1` has `email -> a@x.com` and `sms -> 555-0100`; template `otp` is `Your code is {code}`.

| Situation | `send("u1", "security", "otp", {code: "42"}, [email, sms])` | Gateway calls |
|---|---|---|
| gateway accepts everything | `DELIVERED:email` | `(email, a@x.com, "Your code is 42")` |
| email refused twice, then accepted | `DELIVERED:email` | email, email, email |
| email always refused | `DELIVERED:sms` | email, email, email, sms |
| email and sms always refused | `FAILED` | 6 calls (3 + 3) |
| `[sms, email]` requested instead | `DELIVERED:sms` | `(sms, 555-0100, "Your code is 42")` only |

Checks that never reach the gateway:

```
send("ghost","security","otp",{code:"1"},[email])      -> UNKNOWN_USER   (never registered)
send("u1","security","otp",{},[email])                 -> INVALID        (placeholder {code} has no parameter)
send("u1","security","nope",{},[email])                -> INVALID        (unknown template)
send("u1","security","otp",{code:"1"},[])              -> INVALID        (empty channel list)
set_preference("u1","email","marketing",false)
send("u1","marketing","otp",{code:"1"},[email])        -> SUPPRESSED     (only channel is switched off)
send("u1","security","otp",{code:"1"},[fax])           -> SUPPRESSED     (no address for fax)
send("u1","marketing","otp",{code:"1"},[email, sms])   -> DELIVERED:sms  (email skipped for marketing only)
```

Rendering, with template `echo` = `{a}-{a}-{b}`: params `{a:"1", b:"2", unused:"9"}` give `1-1-2`. With template `{a} {b}` and params `{a:"{b}", b:"x"}` the result is `{b} x`: the `{b}` that came from a value is not replaced again.

Channel listed twice: `[email, email, sms]` with email always refused makes calls email, email, email, sms (email is not retried a second time).

## Edge cases

- Registering a user again replaces all their addresses (old ones are gone) but keeps their opt-out preferences.
- `set_preference` for a user who was never registered is silently ignored; registering that user later does not apply it.
- Preferences are per user: switching email off for `u1` does not affect `u2`.
- Defining a template again replaces it; the next `send` uses the new body.
- Attempt counts restart with every `send`: a channel that failed three times in one send gets three fresh attempts in the next.
- Text in braces that is not a valid placeholder (for example `{}` or `{a b}`) is plain text and needs no parameter.

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- The gateway: `deliver(channel, address, body)` returns `true` when the attempt was accepted and `false` when it was refused. It is provided by the caller of your factory; do not modify it.
- `registerUser(userId, addressesByChannel)`; `setPreference(userId, channel, category, enabled)` (ignored for users that were never registered; nothing is stored for them); `defineTemplate(templateId, body)`.
- `send(userId, category, templateId, params, channels)` returns exactly one of these strings:
  - `UNKNOWN_USER`: the user was never registered.
  - `INVALID`: the template is unknown, the template has a placeholder without a parameter, or the channel list is empty.
  - `SUPPRESSED`: no channel was eligible (none has an address and is enabled for the category), so nothing was attempted.
  - `DELIVERED:<channel>`: an attempt on that channel was accepted.
  - `FAILED`: eligible channels existed but every attempt on every one of them was refused.
- Checks happen in the order `UNKNOWN_USER`, `INVALID`, `SUPPRESSED`, and each of them makes zero gateway calls. Attempt counts restart with every `send`.

## Test entry point (Rust)

The tests drive your system through the thin, primitive-typed contract below (`src/contract.rs`). It is
**not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up
in `src/lib.rs`.

```rust
//! Thin test entry point. Your domain model lives behind it.

/// Provided by the caller (external system). Do not modify.
pub trait DeliveryGateway: Send + Sync {
    /// One delivery attempt. `true` when accepted, `false` when refused.
    fn deliver(&self, channel: &str, address: &str, body: &str) -> bool;
}

pub trait NotificationApi {
    fn register_user(&mut self, user_id: &str, addresses: &[(&str, &str)]);

    /// Ignored for unregistered users. Everything is enabled by default.
    fn set_preference(&mut self, user_id: &str, channel: &str, category: &str, enabled: bool);

    fn define_template(&mut self, template_id: &str, body: &str);

    /// Returns `UNKNOWN_USER`, `INVALID`, `SUPPRESSED`, `FAILED` or `DELIVERED:<channel>`.
    /// Channels are tried in the given order.
    fn send(
        &mut self,
        user_id: &str,
        category: &str,
        template_id: &str,
        params: &[(&str, &str)],
        channels: &[&str],
    ) -> String;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(_gateway: Arc<dyn DeliveryGateway>) -> Option<Box<dyn NotificationApi>> {
    None
}
```

## Language notes

`DeliveryGateway` is a `Send + Sync` trait defined in `src/contract.rs`; `create` receives it as `Arc<dyn DeliveryGateway>`. `NotificationApi` is `&mut self`. The trait methods take string slices; addresses and parameters are passed as slices of `(key, value)` pairs.

## Running

```bash
cargo test -p p23_notification_system
STRICT=1 cargo test -p p23_notification_system   # fail instead of skip while unimplemented
cargo test -p p23_notification_system --features hidden   # interviewer scenarios
cargo fmt --check && cargo clippy -p p23_notification_system --all-targets
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

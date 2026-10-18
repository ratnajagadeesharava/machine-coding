# 23 · Notification System (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 90 min | Level 2 — minimal external API. The public interface is part of the problem; everything behind it is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Build the core of a notification service. Application code asks it to notify a user about something, giving a message
template and the channels it would like to use in order of preference. The service renders the message, respects what the
user has opted out of, tries the channels one by one and tells the caller what happened. The actual delivery to the
outside world (email, SMS, push...) is performed by a **delivery gateway** that is given to you; it is the only
outside dependency.

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

```
registerUser("u1", {email: "a@x.com", sms: "555"})      defineTemplate("otp", "Your code is {code}")
send("u1", "security", "otp", {code: "42"}, [email, sms])      -> DELIVERED:email      gateway: ("email", "a@x.com", "Your code is 42")
(email always refused)                                          -> DELIVERED:sms        gateway: email x3, then sms x1
send("u1", "security", "otp", {}, [email])                      -> INVALID              gateway: no calls
```

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

## Test entry point (Java)

The tests drive your system through the thin, primitive-typed contract below. It is **not** your domain
model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in
`solution/Solution.java`.

```java
package contract;

/** Provided by the caller (external system). Do not modify. */
public interface DeliveryGateway {

    /** One delivery attempt. True when accepted, false when refused. */
    boolean deliver(String channel, String address, String body);
}
```

```java
package contract;

import java.util.List;
import java.util.Map;

/** Thin test entry point. Your domain model lives behind it. */
public interface NotificationApi {

    void registerUser(String userId, Map<String, String> addressesByChannel);

    /** Ignored for unregistered users. Everything is enabled by default. */
    void setPreference(String userId, String channel, String category, boolean enabled);

    void defineTemplate(String templateId, String body);

    /**
     * Returns UNKNOWN_USER, INVALID, SUPPRESSED, FAILED or DELIVERED:&lt;channel&gt;.
     * Channels are tried in the given order.
     */
    String send(String userId, String category, String templateId, Map<String, String> params, List<String> channels);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    NotificationApi create(DeliveryGateway gateway)
}
```

## Language notes

`DeliveryGateway` is given: it lives in the `contract` package next to `NotificationApi`. The tests pass their own implementation to `Solution.create(gateway)`. Keep the gateway out of your domain model: only the sending flow talks to it.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/23-notification-system test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

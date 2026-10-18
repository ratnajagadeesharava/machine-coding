# Interviewer notes — 23 Notification System

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- Parameter values that contain `{other}` must not be expanded again.
- Braces that do not form a placeholder (`{}`, `{not valid}`) are literal text.
- A skipped (disabled) channel between failing and succeeding ones costs no gateway calls.
- `setPreference` for an unregistered user must not leak into a later registration.
- Order of checks: unknown user beats invalid beats suppressed.

## Likely interviewer questions
- How do you model channels: a string, an enum, one class per channel? What does adding a new channel cost? What would per-channel formatting (SMS length limit, HTML for email) do to your design?
- Where does the retry rule live? Could it differ per channel or per category tomorrow?
- Where do preferences live, and what is the lookup key? What does "default enabled" imply for storage?
- How do you make the gateway testable and what would you mock in a unit test of the send flow?
- How would you add an asynchronous variant (V3) without rewriting the send logic?
- Why is the result a string here and what type would you use inside your own code?

## Invariants
- The gateway is called only with an address that belongs to the user and a channel that is enabled for the category.
- The number of gateway calls for one send is at most 3 x (number of distinct eligible channels).
- A send that returns `DELIVERED:c` made exactly one accepted call on channel `c` and none afterwards.
- Refused requests (`UNKNOWN_USER`, `INVALID`, `SUPPRESSED`) cause zero gateway calls.

## Common mistakes
- Retrying 3 times *in total* instead of 3 attempts per channel.
- Rendering the template again for each attempt (or rendering before checking that every placeholder is available).
- Sequential `String.replace` per parameter, which re-expands values that look like placeholders.
- Treating "no address" and "disabled" differently from each other, or returning `FAILED` when nothing was attempted.
- Attempt counter kept in the service state and carried over to the next send.
- Duplicated channel in the request tried twice.
- Storing preferences as "disabled list" mixed with channel lists so that re-enabling does not work.

## Over-engineering traps
- `NotificationChannel` interface with `EmailChannel`, `SmsChannel`, `PushChannel`, a `ChannelFactory`, `RetryStrategy`, `TemplateEngine`, `PreferenceService`, `NotificationEvent` with observers. For this contract: a few maps, one render function and one loop are the core.
- Building a plugin system for channels the gateway already abstracts.

## Trade-offs worth probing
- Strings for channels keep the system open to new channels without code changes; enums give safety but couple the service to the list of channels.
- A retry loop inside `send` is simple; extracting a retry policy pays off only when different channels need different rules.

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.

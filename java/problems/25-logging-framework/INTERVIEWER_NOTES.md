# Interviewer notes — 25 Logging Framework

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## Hidden edge cases (see `interviewer/`)
- `app.db.pool` inheriting from `app` through a missing `app.db`; a sibling `app.dbx` not inheriting from `app.db`.
- Changing an ancestor's level after the child has already logged: the child must see the new threshold (no cached copies).
- Adjacent placeholders `{}{}`; a template with no placeholders but arguments; stray braces.
- An argument containing `{}`.
- Removing and re-adding an appender under the same name; an appender added after earlier messages never sees them.

## Likely interviewer questions
- How do you resolve the effective level — walk up the name each time, cache per logger and invalidate, or build an explicit tree? Cost per call? Cost of `setLevel`?
- Where is the "is this enabled" check relative to message formatting? Why does it matter at 1M msgs/s?
- Is the logger a first-class object (`getLogger("app")`) or just a name string in a map? What changes if the API hands out logger objects?
- How do you keep one failing sink from breaking the rest — and should the failure be reported somewhere?
- How would you add a second output format (V2) without touching `log`?
- What would change if `log` were called from many threads (copy-on-write appender list? lock-free reads?)

## Invariants
- Effective level of a logger is always the level of its nearest configured ancestor-or-self (or the root's).
- A message is formatted at most once, and only if at least one appender will receive it (ideal) — at minimum, only if it passes the logger threshold.
- Appender order of delivery is registration order.
- A sink failure never changes delivery to other sinks.

## Common mistakes
- Treating `application` as a child of `app` (`startsWith` instead of segment-aware lookup).
- Caching the effective level on first use and never invalidating it when an ancestor changes.
- Substituting placeholders by repeated `replaceFirst`/`replace`, which re-scans argument text (`"{}"` as an argument corrupts later placeholders) and mishandles regex characters.
- Filtering appenders by level but forgetting the logger threshold (or the other way round).
- Formatting the message before the level check.
- Letting an exception from one sink propagate (or swallowing it in a way that skips later sinks).
- Comparing levels as strings instead of by rank.
- Using a wall clock instead of the injected clock for the timestamp.

## Over-engineering traps
- `Logger`, `LoggerFactory`, `LogManager`, `Formatter`, `Layout`, `Filter`, `Handler`, `LogRecord`, `Level` hierarchies with a plugin loader — before one test needs more than a map from name to level and a list of sinks.
- Async queues, rolling files and MDC in V1.

## Scalability / concurrency discussion
- Hot path: level lookup. A cache keyed by logger name with a global "generation" counter invalidated by `setLevel` is simple and fast.
- Appender list as an immutable snapshot swapped on add/remove so `log` never takes a lock.
- Pre-sizing the message buffer, avoiding `String.format`.

## Trade-offs worth probing
- Explicit logger tree vs. string-based lookup with a cache.
- Eager vs. lazy message formatting (lambdas/suppliers) for expensive arguments.

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden` (Java) / `cargo test --features hidden` (Rust). They only use the same public contract.

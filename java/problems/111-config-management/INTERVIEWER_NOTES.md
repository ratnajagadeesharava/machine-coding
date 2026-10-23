# Interviewer notes — 111 Configuration Management Service

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- `update` with an invalid value and a stale version: the answer is the validation error, not `false`; with an invalid key on a missing config: also the validation error.
- Rolling back twice: rolling back to a version that was itself a rollback; old versions keep their original timestamps, new ones get the current clock.
- Values that look like the output format (`a/b=c v1 t2`, newlines, `=`, Unicode, leading/trailing spaces) are stored and returned verbatim.
- Updates and rollbacks racing on one key: versions stay 1..N with no gaps or duplicates and `get` equals the last history entry.
- `list` taken while other threads create keys: sorted, never throws, never shows a half-created record.
- Delete, create and update racing on one key: no exception other than the documented ones, and history versions restart cleanly at 1 after a delete.
- 5,000 versions on one key: `get` and `list` must not walk the history.
- `list` order is plain string order (also pinned by a visible test): `1st` < `alpha` < `mid-key` < `mid.key` (`-` is 0x2D, `.` is 0x2E).

## Likely interviewer questions
- Where is the version check performed, and what makes check-and-write atomic?
- Why is a stale version `false` while a missing config throws? Would you return a richer result type in production?
- `history` includes the current version: what would change if it did not?
- What is the lock granularity: global, per namespace, per key? What does `list` have to lock?
- Why does rollback add a version instead of moving the pointer back? What would break in an audit trail?
- Should `update` with an unchanged value bump the version? What are the arguments each way (watchers, caches, ETags)?

## Invariants
- A config's versions in its history are exactly 1..current, in order, each with the timestamp of its own write.
- The current record is always the last history entry.
- A failed call (invalid, duplicate, missing, stale) changes no state and consumes no version number.
- Validation order: arguments, then existence, then version comparison.

## Common mistakes
- Checking the version, then writing in a separate step (two winners under contention); or using a counter that is incremented even when the compare fails.
- Validating the value after the version check, or after mutating (a bad update bumps the version or appends history).
- Returning the internal history list or a live view from `history`/`list`.
- Rollback that rewrites an old record, truncates history, or reuses the old version number.
- Sorting `list` with a locale-aware comparator, or sorting by insertion order.
- A global lock held while building strings, or a `list` that locks every key one by one and returns an inconsistent mix.
- Treating `delete` as a soft delete (history survives and the next create continues at version N+1).
- Using the wall clock instead of the injected clock, or reading it before taking the lock so timestamps go backwards relative to versions.
- Regex `[a-z0-9._-]` written with `\w` or `.` unescaped, accepting uppercase or spaces; `String.isBlank` vs `isEmpty` mixups for namespace.

## Over-engineering traps
- An event bus, a plugin system or a repository interface with two implementations before any requirement needs them.
- A class hierarchy of "config types", or a generic `Config<T>`, when values are strings.
- Persisting to disk, or building an expression language for values.

## Design and scaling discussion
- Per-key records with an immutable history list: updates swap a new record in atomically (compare-and-swap on the map entry or a per-key lock); reads never block.
- `list` consistency: per-key snapshots are enough for this contract, a namespace-wide consistent snapshot needs a namespace lock or versioned snapshots (V5).
- Version numbers are the foundation of everything later: ETag-style conditional requests, replication cursors, cache invalidation.
- Memory: unbounded history is a leak in production; V4 bounds it, which forces a decision about stable version numbers.
- Rollback creating a new version keeps the history linear and auditable and makes watchers' job simple (every change is a forward event).

## Trade-off questions to ask when the candidate is done
- Per-key lock vs one lock per namespace vs a concurrent map with compute: what does each cost when `list` runs?
- Would you store views (strings) or typed records internally, and where does the string get built?
- How would you add `watch(namespace)` without making writers wait on slow listeners?
- What breaks if two services use different clocks, and why is `updatedAt` informational and the version authoritative?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.

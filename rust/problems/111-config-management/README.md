# 111 · Configuration Management Service (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 60 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## What you are building

Applications read their settings (timeouts, endpoints, limits) from a central configuration service instead of baking them into the deployment. Each setting is a **config**: a string value stored under a `namespace` (for example `app/web`) and a `key` (for example `db.host`). Operators change values while services keep reading them, so the service keeps a version number for every config, refuses edits based on stale information, remembers every earlier version, and can put an old value back. You are building that service behind a small, primitive-typed entry point.

## Requirements

1. A config is identified by (`namespace`, `key`) and carries a string `value`, a `version` and an `updated_at` timestamp. Namespaces are case-sensitive; the same key may exist in many namespaces independently.
2. `create(namespace, key, value)` stores a new config at version 1 with `updated_at` = the injected clock's current epoch milliseconds. Creating a config that already exists is an error and changes nothing.
3. `update(namespace, key, value, expected_version)` replaces the value only when the config's current version equals `expected_version`. On success the version goes up by exactly 1 (even if the new value equals the old one), `updated_at` = now, and the call returns `true`. When the versions differ it returns `false` and changes nothing. Updating a config that does not exist is an error (not `false`).
4. `get(namespace, key)` returns the current record, or `None` if there is none.
5. `delete(namespace, key)` removes the config together with its history and returns whether it existed. Creating it again starts over at version 1 with a fresh history.
6. `list(namespace)` returns the current record of every config in the namespace, ascending by key (plain string order). An unknown namespace gives an empty list.
7. `history(namespace, key)` returns every version the config has had, oldest first, **including the current one**. Each entry keeps the value, version and `updated_at` it had when it was written. A missing config gives an empty list. The returned list is a snapshot: later changes never alter it.
8. `rollback(namespace, key, to_version)` creates a **new** version (current + 1, `updated_at` = now) whose value equals the value `to_version` had; earlier history is untouched. Rolling back to the current version is allowed and still creates a new version. It returns the new version number. A `to_version` that never existed for this config (below 1 or above the current version) is an error; rolling back a missing config is an error.
9. Validation: a blank (empty or whitespace-only) namespace is invalid; a key must be non-empty and use only the characters `[a-z0-9._-]`; a value must be at most 256 characters (an empty value is valid). Every operation validates the namespace and key it is given (`list` only the namespace), `create` and `update` also validate the value.
10. Validation happens before anything else (before existence and version checks). A call that fails for any reason changes nothing — no partial updates, no version bump, no history entry.
11. The service is called from many threads at once and must stay correct (see the concurrency cases below).

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- A record is reported as one string, a **view**: `<namespace>/<key>=<value> v<version> t<updated_at>`, e.g. `app/db.host=localhost v2 t1500`. `updated_at` is `clock.now_millis()` at the moment that version was written. The value is inserted verbatim, whatever characters it contains.
- `Result<_, ConfigError>`: `InvalidArgument` for invalid input and for a `to_version` that never existed, `AlreadyExists` when `create` finds the config already present, `NotFound` when `update` or `rollback` finds no such config. Versions are `u64`. `update` returns `Ok(true)` / `Ok(false)`; `get` returns `Ok(Option<String>)`.
- The clock is injected when the service is created; the tests advance a fake clock and never sleep.
- Order of checks for `update`: invalid input first, then missing config, then the version comparison.
- A stale `expected_version` (older, newer, or 0) is `Ok(false)`, never an error.

## Examples

```
clock = 1000
create("app/web", "db.host", "localhost")  -> Ok(())
get("app/web", "db.host")      -> Ok(Some("app/web/db.host=localhost v1 t1000"))

clock = 1500
update("app/web", "db.host", "10.0.0.5", 1)   -> Ok(true)
update("app/web", "db.host", "10.0.0.9", 1)   -> Ok(false)   (version is 2 now; nothing changed)
get("app/web", "db.host")      -> Ok(Some("app/web/db.host=10.0.0.5 v2 t1500"))
history("app/web", "db.host")  -> Ok(["app/web/db.host=localhost v1 t1000",
                                      "app/web/db.host=10.0.0.5 v2 t1500"])
```

```
clock = 2000
rollback("app/web", "db.host", 1)  -> Ok(3)
get("app/web", "db.host")          -> Ok(Some("app/web/db.host=localhost v3 t2000"))
history(...)                       -> v1 localhost t1000, v2 10.0.0.5 t1500, v3 localhost t2000
```

```
create("app", "zeta", "z"); create("app", "alpha", "a"); create("app", "mid-key", "m")
list("app")       -> Ok(["app/alpha=a v1 t..", "app/mid-key=m v1 t..", "app/zeta=z v1 t.."])
list("nobody")    -> Ok([])
```

```
create("app", "Bad Key", "x")              -> Err(InvalidArgument)
create("  ", "k", "x")                     -> Err(InvalidArgument)
create("app", "k", <257 characters>)       -> Err(InvalidArgument)
create("app", "k", "1"); create("app", "k", "2")   -> Err(AlreadyExists)
update("app", "k", <257 characters>, 99)   -> Err(InvalidArgument)  (validation wins over the version check)
update("app", "missing", "x", 1)           -> Err(NotFound)
rollback("app", "k", 5)  (current is 1)    -> Err(InvalidArgument)
```

## Edge cases

- Concurrent `create` of the same config: exactly one caller succeeds, the rest get the "already exists" error.
- 16 threads calling `update` with the same `expected_version`: exactly one returns `true`; the version ends one higher.
- 8 threads that each re-read the version and retry `update` until it succeeds, 250 times each, lose no update: the final version is 1 + 2,000 and the history holds versions 1..2001 with no gaps or duplicates.
- Versions stay contiguous (1, 2, 3, ...) in the history even when updates and rollbacks on one key happen at once.
- `list` is always sorted by key, also while other threads are creating configs.
- Values may contain any characters (spaces, `=`, `/`, newlines, Unicode, text that looks like a view); the length limit counts characters.
- Deleting and re-creating a config does not carry over any history or version.
- Keys are compared exactly: `a.b`, `a_b` and `a-b` are three different keys.

## Glossary

- **Config**: one named setting, identified by namespace and key.
- **Namespace**: a free-form grouping such as `app/service`; case-sensitive, only required to be non-blank.
- **Version**: starts at 1 and goes up by 1 on every change (update or rollback); never reused for the same config until it is deleted.
- **Expected version**: the version the caller believes is current; the update is refused if it is not.
- **History**: all versions of a config, oldest first, the current one last.
- **Rollback**: writing an old value back as a new version; it does not rewrite or remove history.
- **View**: the one-string form of a record used by the entry point.

## Minimum API the tests expect (Rust)

The tests drive your system through the thin contract below (`src/contract.rs`). It is **not** your domain model — it is the doorway. Add your own modules/types/traits next to it and wire them up in `src/lib.rs`.

```rust
//! Thin test entry point. Your design lives behind it.
//! Must be usable from multiple threads (`&self`, `Send + Sync`).
//!
//! A config record is reported as a single string (a "view"):
//! `<namespace>/<key>=<value> v<version> t<updated_at>`, where `updated_at` is the injected clock's epoch
//! milliseconds at the moment that version was written. Example: `app/db.host=localhost v2 t1500`.
//!
//! Argument validation always happens first, before existence and version checks, and a failed call changes nothing.

#[derive(Debug, PartialEq, Eq)]
pub enum ConfigError {
    /// Blank namespace; empty key or key with a character outside `[a-z0-9._-]`; value longer than 256
    /// characters (an empty value is allowed); rollback to a version that never existed for the config
    /// (below 1 or above the current version).
    InvalidArgument,
    /// `create` of a (namespace, key) that already exists.
    AlreadyExists,
    /// `update` or `rollback` of a config that does not exist.
    NotFound,
}

pub trait ConfigService: Send + Sync {
    /// Creates a config at version 1 with `updated_at` = now.
    fn create(&self, namespace: &str, key: &str, value: &str) -> Result<(), ConfigError>;

    /// Replaces the value if and only if the current version equals `expected_version`; the version then goes up
    /// by exactly 1 (even when the new value equals the old one) and `updated_at` = now. `Ok(true)` on success.
    /// `Ok(false)`, changing nothing, when the current version differs from `expected_version`.
    fn update(
        &self,
        namespace: &str,
        key: &str,
        value: &str,
        expected_version: u64,
    ) -> Result<bool, ConfigError>;

    /// The view of the current version, or `None` if the config does not exist.
    fn get(&self, namespace: &str, key: &str) -> Result<Option<String>, ConfigError>;

    /// Removes the config together with its history; `Ok(true)` if it existed. Creating it again starts over at
    /// version 1.
    fn delete(&self, namespace: &str, key: &str) -> Result<bool, ConfigError>;

    /// Views of the current version of every config in the namespace, ascending by key (plain string order).
    /// Empty for an unknown namespace.
    fn list(&self, namespace: &str) -> Result<Vec<String>, ConfigError>;

    /// Views of every version the config has had, oldest first, including the current one (the last element equals
    /// what `get` returns). Empty if the config does not exist. The result is a snapshot.
    fn history(&self, namespace: &str, key: &str) -> Result<Vec<String>, ConfigError>;

    /// Creates a NEW version (current version + 1, `updated_at` = now) whose value is the value that `to_version`
    /// had. Earlier history is untouched. Rolling back to the current version is allowed and still creates a new
    /// version. Returns the new version number.
    fn rollback(&self, namespace: &str, key: &str, to_version: u64) -> Result<u64, ConfigError>;
}
```

Your wiring point (`src/lib.rs`):

```rust
pub fn create(clock: Arc<dyn Clock>) -> Option<Arc<dyn ConfigService>> { ... }
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Time comes from an injected fake clock, never real sleeping. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Create, get, duplicate create, missing configs, and the exact view format with timestamps from the clock.
- Optimistic update: matching version, stale / future / zero version, identical value still bumps, update of a missing config.
- List ordering, namespace isolation and case sensitivity, unknown namespaces.
- History order and snapshot behaviour, rollback (new version, current version, bad version, missing config).
- Delete and re-create, validation of namespace, key charset and value length on every operation, no partial change on failure.
- Concurrency: one winner among 16 creators, one winner among 16 updaters with the same version, no lost updates under retry loops, parallel creates across namespaces.

## Running

```bash
cargo test -p p111_config_management
STRICT=1 cargo test -p p111_config_management   # fail instead of skip while unimplemented
cargo test -p p111_config_management --features hidden   # interviewer scenarios
cargo play run 111              # from the repo root
```

Until `create` returns `Some(..)` every test prints `SKIPPED` and passes, so a fresh checkout builds green. Test code: `tests/behavior.rs`. Shared helpers (`FakeClock`, `sut!`): `rust/support`.

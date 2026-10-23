# 52 · Singleton (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 30 min | Level 2 — one stub type, `AppConfig`, in `src/lib.rs`. |

## Problem

**What you are building.** Every part of the application reads and writes one shared configuration object (think of
settings like `mode=fast`). All callers must see the very same object, so a value set in one place is visible everywhere.
Creating it is expensive, so it must be created only when first needed, and never twice — even when 16 threads ask for it
at the same instant.

**Glossary**
- *Instance* — the single `AppConfig` shared by the whole process, handed out as an `Arc<AppConfig>`.
- *Lazy* — not created at start-up, only on the first `instance()` call.
- *Construction* — one creation of an `AppConfig` value; `instances_created()` counts these.

## Requirements

1. `AppConfig::instance()` always returns the same object (an `Arc<AppConfig>`; the tests compare with `Arc::ptr_eq`) until `reset_for_tests()` is called.
2. **Lazy:** nothing is created before the first `instance()`. `instances_created()` is 0 before, 1 after the first call (and stays 1 on further calls).
3. **Thread-safe creation:** 16 threads call `instance()` at the same moment (on a barrier), 300 rounds in a row with a reset between rounds: every thread gets the same object and `instances_created()` is exactly 1 each round.
4. Nobody else can construct one: the only way to get an `AppConfig` is `AppConfig::instance()`. The type has no `pub fn new`, no `Default`, no `Clone` and no public fields — the test scans the non-comment lines of `src/lib.rs` for the words `pub fn new`, `Default` and `Clone`, and for `pub ...,` field lines.
5. `set(&self, key, value)` stores an entry; `get(&self, key) -> Option<String>` returns the value, or `None` if the key was never set. Setting an existing key replaces its value. Both are safe from many threads: after 32 threads each write 100 distinct keys, all 3200 values are present and correct.
6. `reset_for_tests()` (a test hook) drops the current instance and zeroes the counter. The next `instance()` creates a **new** object with none of the old entries, and `instances_created()` is then 1 again.

## Examples

| Steps | Result |
|---|---|
| `reset_for_tests()`; `instances_created()` | `0` |
| `instance()` twice | `Arc::ptr_eq` is true; `instances_created()` = `1` |
| `instance().set("mode","fast")`, later `instance().get("mode")` | `Some("fast")` |
| `instance().get("missing")` | `None` |
| `a = instance(); a.set("a","1"); reset_for_tests(); b = instance()` | not `ptr_eq`; `b.get("a")` is `None`; `instances_created()` = `1` |

## Edge cases

- 16 threads racing on the very first `instance()` must not produce two objects, even briefly.
- Concurrent `set` calls on different keys must not lose any entry.
- The tests run one at a time against the process-wide instance, so state left by one test must be clearable with `reset_for_tests()`.

## Running

```bash
cargo test -p p52_singleton     # from rust/ — tests FAIL until you implement AppConfig
cargo play run 52
```

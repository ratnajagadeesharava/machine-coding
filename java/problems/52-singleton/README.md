# 52 · Singleton (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 30 min | Level 2 — one stub class, `AppConfig`. |

## Problem

**What you are building.** Every part of the application reads and writes one shared configuration object (think of
settings like `mode=fast`). All callers must see the very same object, so a value set in one place is visible everywhere.
Creating it is expensive (imagine it loads files), so it must be created only when first needed, and never twice — even
when 16 threads ask for it in the same microsecond.

**Glossary**
- *Instance* — the single `AppConfig` object shared by the whole process.
- *Lazy* — not created at class-loading or start-up, only on the first `getInstance()`.
- *Constructor run* — one execution of the `AppConfig` constructor; `instancesCreated()` counts these.

## Requirements

1. `AppConfig.getInstance()` always returns the same object (`==` identical) until `resetForTests()` is called.
2. **Lazy:** nothing is created until the first `getInstance()`. `instancesCreated()` is 0 before, 1 after the first call (and stays 1 on further calls).
3. **Thread-safe creation:** when many threads call `getInstance()` simultaneously, exactly one instance is created (`instancesCreated()` is 1) and every thread gets that same object. (The test starts 16 threads together on a barrier, 300 rounds in a row, resetting between rounds.)
4. Nobody outside the class can create an instance or subclass it (`AppConfig` is a `final` class with only private constructors).
5. `set(key, value)` stores a configuration entry; `get(key)` returns `Optional<String>` — the value, or `Optional.empty()` if the key was never set. Setting an existing key replaces its value. Both are safe to call from many threads at once: after 32 threads each write 100 distinct keys, every one of the 3200 values is present and correct. A `null` key in `set` or `get` → `IllegalArgumentException`.
6. `resetForTests()` (a test hook) drops the current instance and sets the counter back to 0. The next `getInstance()` creates a **new** object that has none of the old entries, and `instancesCreated()` is then 1 again.

## Examples

| Steps | Result |
|---|---|
| `resetForTests()`; `instancesCreated()` | `0` |
| `getInstance()` twice | same object both times; `instancesCreated()` = `1` |
| `getInstance().set("mode","fast")`, later `getInstance().get("mode")` | `Optional["fast"]` |
| `getInstance().get("missing")` | `Optional.empty` |
| `getInstance().set(null,"x")` / `get(null)` | `IllegalArgumentException` |
| `a = getInstance(); a.set("a","1"); resetForTests(); b = getInstance()` | `b != a`; `b.get("a")` empty; `instancesCreated()` = `1` |

## Edge cases

- 16 threads racing on the very first `getInstance()` must not produce two objects, even briefly.
- Concurrent `set` calls on different keys must not lose any entry.

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement AppConfig
```

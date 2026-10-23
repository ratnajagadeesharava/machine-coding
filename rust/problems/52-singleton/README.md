# 52 · Singleton (Rust)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 30 min | Level 2 — one stub type, `AppConfig`, in `src/lib.rs`. |

## Problem

Every part of the application reads and writes one shared configuration object. Creating it is expensive, so it
must be created only when first needed, and never twice — even when 16 threads ask for it at the same instant.

## Requirements

1. `AppConfig::instance()` always returns the same object (an `Arc<AppConfig>`; the tests compare with `Arc::ptr_eq`).
2. **Lazy:** nothing is created before the first `instance()`. `instances_created()` is 0 before, 1 after (and stays 1).
3. **Thread-safe:** 16 threads call `instance()` at the same moment, 300 rounds in a row: exactly one instance is created each round.
4. Nobody else can construct one: the only way to get an `AppConfig` is `AppConfig::instance()` (no `pub fn new`, `Default`, `Clone` or public fields; the tests scan `src/lib.rs` for these).
5. `set` / `get` store configuration and are safe from many threads (32 writer threads in the test).
6. `reset_for_tests()` drops the instance and zeroes the counter.

## Running

```bash
cargo test -p p52_singleton     # from rust/ — tests FAIL until you implement AppConfig
cargo play run 52
```

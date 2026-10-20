# 52 · Singleton (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium | 30 min | Level 2 — one stub class, `AppConfig`. |

## Problem

Every part of the application reads and writes one shared configuration object. Creating it is expensive
(imagine it loads files), so it must be created only when first needed, and never twice — even when 16 threads ask
for it in the same microsecond.

## Requirements

1. `AppConfig.getInstance()` always returns the same object.
2. **Lazy:** nothing is created until the first `getInstance()`. `instancesCreated()` is 0 before, 1 after (and stays 1).
3. **Thread-safe:** when many threads call `getInstance()` simultaneously, exactly one instance is created and everybody gets it. (The test starts 16 threads together, 300 times in a row.)
4. Nobody outside the class can create an instance or subclass it.
5. `set(key, value)` / `get(key)` store configuration; they are safe to call from many threads at once (32 writer threads in the test). `null` keys → `IllegalArgumentException`.
6. `instancesCreated()` counts constructor runs since the last `resetForTests()`; `resetForTests()` drops the instance and zeroes the counter. (A test hook.)

## Running

```bash
mvn test            # from this directory — tests FAIL until you implement AppConfig
```

# Follow-ups — 122 Versioned Value Store

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Range scans at a version
`scan(fromKey, toKey, version)` returns the live key/value pairs in key order as of a version (or of a snapshot), and a `limit`/cursor makes it pageable. Which part of your data layout makes this cheap, and what happens to a scan that runs while writers and a compaction are active?

### V3 — Time-to-live
`put(key, value, ttlMillis)` makes a value disappear after a duration, measured with an injected clock. Is expiry a write (does it consume a version and show up in `history`) or only a rule applied when reading? What does `getAt` for a version from before the expiry say, and what does a snapshot say?

### V4 — Time travel by timestamp
`getAsOf(key, timestamp)` answers with the value the key had at a wall-clock time. How do you map time to versions when the clock can stand still or step backwards, and how do you keep that mapping small after compaction?

### V5 — Multi-key transactions with conflict detection
`begin()` reads at a version, buffers writes, and `commit()` succeeds only if none of the keys it read or wrote were changed after its read version; a successful commit is ONE version, like `putAll`. What exactly must be checked and held atomically at commit, how do you avoid starving big transactions, and how does this relate to `putIfVersion`?

### V6 — Persistence and garbage collection
Every write is appended to a log before it is acknowledged, a restart rebuilds the store from the log, and compaction runs by itself under a policy (memory budget, age of the oldest open snapshot). What goes into a log record, how does recovery find the right version counter, and what do you do about a snapshot that someone forgot to release?

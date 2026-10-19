# Follow-ups — 74 Concurrent LRU Cache (Read-Write Lock)

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — TTL
Entries expire after a time-to-live (use an injectable clock). Is expiry done on read, in the background, or both? Which lock?

### V3 — Do reads need the write lock at all?
Record accesses in a lock-free buffer and replay them under the write lock in batches (Caffeine-style). What do you give up in exactness of the LRU order?

### V4 — Striping
Shard the cache into 16 segments by key hash. Each segment has its own lock and its own LRU list. What happens to the global capacity and to 'least recently used' globally?

### V5 — Loader
`getOrLoad(key, loader)` computes missing values at most once per key even under contention. What happens if the loader is slow or throws? (see problem 30)

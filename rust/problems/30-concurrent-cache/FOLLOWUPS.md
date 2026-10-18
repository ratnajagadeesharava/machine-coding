# Follow-ups — 30 Concurrent Cache with Loaders

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Refresh ahead
Serve a slightly stale value while one background refresh runs; callers never wait on a refresh once a value exists.

### V3 — Negative caching and per-entry TTL
Cache loader failures for a short time; let `put` carry its own TTL. How does the eviction order change?

### V4 — Weighted capacity
Entries have a size (bytes); the capacity is a total weight. One large insert may evict several small entries.

### V5 — Statistics and invalidation
Hit/miss/eviction counters, `invalidate(key)` that also cancels or ignores a computation in flight, and `invalidateAll()`.

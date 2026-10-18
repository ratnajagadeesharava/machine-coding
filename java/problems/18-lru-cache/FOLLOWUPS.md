# Follow-ups — 18 LRU Cache

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Thread safety
Many threads share the cache. Where does the lock go, and why is a plain read-write lock not enough for `get`?

### V3 — Expiry
Each entry may carry a time-to-live; expired entries behave as absent. Lazy or eager removal?

### V4 — Pluggable policy
The same cache should also offer least-frequently-used and FIFO eviction. What is the smallest change that supports it?

### V5 — Loading cache
A miss triggers a loader function; concurrent misses for the same key must load only once.

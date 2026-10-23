# Follow-ups — 118 Search Result Cache

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — LFU variant
Add a least-frequently-used eviction policy (ties broken by recency) that can be chosen when the cache is created. What has to change in your entry bookkeeping, and what is the cost of an eviction? How do you keep one-hit wonders from evicting a hot entry forever, or an old hot entry from living forever?

### V3 — Negative caching
Backend failures "not found / bad query" that the backend marks as permanent are remembered for a short TTL (for example 5 s), while transient failures are still never cached. Where does the distinction live? What does a hit on a cached failure count as in the statistics, and does a coalesced failure change?

### V4 — Stale-while-revalidate
For a short grace period after the TTL (or after an index change), serve the stale result immediately and refresh it in the background, with exactly one refresh in flight per query. What does `size()` mean now, what do the statistics count, and what happens if the refresh fails or the index changes again?

### V5 — Per-query TTL
Some queries (for example "trending") need 10 s, others 10 min. Who decides the TTL: the caller, a rule on the query, or the backend response? How does it interact with LRU and with expiry-based `size()`; do you need a faster way to find expired entries than scanning?

### V6 — Many nodes
Ten cache nodes sit in front of one backend. How does an index change reach all of them in a bounded time, and how do you stop ten nodes from each coalescing separately into ten backend calls (a shared tier, request routing by key)?

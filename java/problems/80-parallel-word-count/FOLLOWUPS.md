# Follow-ups — 80 Parallel Word Count

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Top-K
Return the K most frequent words (ties alphabetical). Do you need the full merged map, or can you keep partial heaps?

### V3 — Streaming input
Documents arrive from a queue and the totals must be available at any time. How do you snapshot without stopping the counters?

### V4 — Shared concurrent map vs per-thread maps
Implement both (`ConcurrentHashMap.merge` / sharded `Mutex<HashMap>` vs thread-local maps merged at the end). Measure under a hot-key workload.

### V5 — Unicode
Words with accents and apostrophes. What breaks in your split-inside-a-document logic?

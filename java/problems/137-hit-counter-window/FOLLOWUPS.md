# Follow-ups — 137 Configurable Time Window Hit Counter (Java)

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Sub-second precision with bounded memory
Dashboards want "hits in the last 1,000 ms, updated every 100 ms" for a window of one hour on a million keys. Accept a configurable resolution (for example 100 ms) so that hits in the same slot are merged. What exactly does a hit at a slot boundary count as, and how much error does the resolution introduce at the window edge? What does `memoryProbe` become?

### V3 — Per-key windows
Some keys need 10 seconds, others 24 hours. Add a per-key window (default = the counter-wide one). What does `setWindow` mean for keys with their own window, what does `totalInWindow` mean when windows differ, and what happens to the memory budget of the 24-hour keys?

### V4 — Rate limiting on top
Add `tryAcquire(key, limit)` that records a hit only if the key has fewer than `limit` hits in the window, atomically (no two threads may both pass at `limit - 1`). Compare this sliding-window limiter with a fixed-window one and a token bucket: which burst can each one let through, and what does each cost per key?

### V5 — Many counters, one answer
Ten servers each count their own hits and a coordinator needs the global windowed count and top keys. How do you merge counters whose clocks disagree by a few hundred ms, and what do you send (every hit, buckets, a sketch)? Then consider approximate counters (a count-min sketch with expiring slices): what error bound do you accept for `topKeys`, and what can no longer be exact?

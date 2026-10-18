# Follow-ups — 19 LFU Cache

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Aging
Old popularity should fade: counts halve every N operations, or an entry's count is reduced when it has not been used for a while. What does that do to your structure?

### V3 — Thread safety
Many threads share the cache. What is the critical section of `get` now, and can reads be concurrent at all?

### V4 — Admission
A one-hit-wonder should not push out a popular entry (TinyLFU-style admission). How would you decide whether to admit a new key at all?

### V5 — Policy switch
Support LRU and LFU behind the same API with a runtime choice. Which parts of your design were policy-specific?

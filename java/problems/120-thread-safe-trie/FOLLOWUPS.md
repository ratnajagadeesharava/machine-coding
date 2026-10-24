# Follow-ups — 120 Thread-Safe Trie

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Wildcard search
Add `matches(pattern)` returning every word that fits a pattern in which `.` stands for exactly one arbitrary character (`c.t` matches `cat`, `cot`, `c-t`, not `cart`), sorted, with a limit. What is the cost for a pattern that starts with `.`? Does the answer stay a consistent snapshot under concurrent writers, and does your locking scheme make a long wildcard scan hurt writers?

### V3 — Longest common prefix
Add `longestCommonPrefix(prefix)`: the longest string that every word starting with `prefix` begins with (for words `car`, `card`, `care` and prefix `c` the answer is `car`; for a prefix nobody has, the empty string or an error, your call, stated). How does it relate to the structure you store, and how long does it take when there are millions of matches? Is it consistent with a `countWithPrefix` taken at the same moment?

### V4 — Readers that never wait
Measure your design with 15 reader threads and one writer thread. Now make `contains`, `startsWith` and `wordsWithPrefix` not block on writers at all, without ever showing a half-inserted word. What do writers pay for that (allocation, copying, memory reclamation, writer-writer contention)? Which operations are still allowed to take a lock and why?

### V5 — Ten million words in memory
The set grows to 10,000,000 words averaging 20 characters, mostly sharing long prefixes. Estimate your cost per word and per node, then shrink it: merge single-child chains, store children compactly, share storage for characters, drop empty branches after deletes. What do you give up in speed or lock granularity? How would you measure it?

### V6 — Expiry and options
Words can be inserted with a time-to-live and disappear on their own (injected clock, no background thread required for correctness); `size()` and every query must ignore expired words immediately. Also add a case-sensitive mode chosen at creation. What is purged lazily, what eagerly, and how do concurrent readers see a word expire in the middle of a `wordsWithPrefix` call?

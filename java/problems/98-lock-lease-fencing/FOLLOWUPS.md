# Follow-ups — 98 Lease Lock with Fencing Tokens

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Blocking acquire with timeout
Add `acquire(lock, owner, leaseMs, waitMs)` that waits for the lock in FIFO order. How do waiters learn about a release, and what wakes them when a lease merely expires, with no one calling?

### V3 — Replicated lock service
Run the service on three nodes with majority agreement, using the leader election of problem 99. Who issues tokens, and how do you keep them monotonic across a leader change?

### V4 — Clock skew
Client and server clocks disagree by 200 ms and the client's lease timer drifts. How should the client compute how long it may act on a lease, and why do fencing tokens still protect the store?

### V5 — Reentrancy and hierarchy
Let an owner acquire the same lock multiple times (counted) and add lock groups that must be taken in order. What does `release` mean now, and what happens to the token on a nested re-acquire?

### V6 — Garbage collection of locks
A hundred million lock names are used once. When can you forget a lock, given that its token counter must never go backwards for a name that comes back?

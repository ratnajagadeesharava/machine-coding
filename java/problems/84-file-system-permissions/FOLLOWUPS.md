# Follow-ups — 84 In-Memory File System with Permissions

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Symlinks and hard links
Add symbolic links that are resolved during traversal (with a loop limit) and hard links that give one file two names. How do permission checks, delete and move change? Where does a link count live?

### V3 — ACLs and sticky directories
Add per-entry access lists (`user:bob:rw-`) and a sticky bit on `/tmp` so that only the owner can delete or rename their own entries. Where does the permission check become a policy object instead of three bit masks?

### V4 — Quotas and sizes
Each user has a byte quota and every file has a size. Writes that would exceed the quota fail atomically. How do move and chown affect quota accounting, and how do you keep the totals correct under concurrency?

### V5 — Watching and audit
Observers subscribe to a directory and get events for create/delete/rename/chmod, and every denied operation is logged with user, path and required permission. How do you deliver events in order without holding tree locks while calling listeners?

### V6 — Finer-grained locking
A single lock makes every operation serial. Move to per-directory or per-node locking. What is the lock order for a move across two directories, and how do you avoid deadlock and the 'move a directory into its own subtree' race?

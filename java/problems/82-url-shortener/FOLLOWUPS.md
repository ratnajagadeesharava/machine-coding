# Follow-ups — 82 URL Shortener

Open **one version at a time**, only after the previous one passes. Each is designed to stress the architecture you chose.
Tests for follow-ups are **not provided** unless they fit the existing contract — extend the contract/tests yourself;
deciding how to test it is part of the exercise.

### V2 — Expiry
Links get an optional time-to-live using an injectable clock. An expired code behaves like a deleted one but must not be reissued. Is expiry checked on resolve, swept in the background, or both? Does a resolve of an expired code count a click?

### V3 — Per-user links and quotas
Each customer has an id; the same URL shortened by two customers gets two different codes, and each customer may hold at most N active links. How does 'same URL, same code' change, and where is the quota check made atomic?

### V4 — Click analytics
Record clicks per day and top referrers without making `resolve` slow. Which counters can be sharded or buffered and merged later, and what do you tell a caller that reads `clicks` during the merge?

### V5 — Multiple nodes
Run 10 stateless servers over a shared store. How do you hand out codes without two servers issuing the same one (ranges, central counter, hash + collision retry)? How does an alias reservation stay unique?

### V6 — Unguessable codes
Sequential codes leak volume and are enumerable. Generate non-sequential codes of at most 6 characters without collisions. What happens as the code space fills up?

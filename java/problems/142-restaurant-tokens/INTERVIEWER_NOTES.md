# Interviewer notes — 142 Restaurant Customer Tokens

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Two promoted tokens of different priority: promotion is by token number, not by priority (T-1 DINE_IN before T-2 TAKEAWAY).
- A recalled token's pass-over count is reset: it must not stay promoted after coming back, while another promoted token stays ahead of it.
- A recalled token jumps older tokens of its own class but stays behind VIPs (and behind promoted tokens).
- Pass-overs made by a counter that cannot serve the waiting token still count; a token issued before the waiting one never counts.
- Pass-overs after a recall count again from zero (fifth later token promotes it again).
- `expireOverdue` stamps the no-show time with the time of the call (not `calledAt + grace`), and the recall window runs from that stamp.
- `NO_SHOW` stays `NO_SHOW` after the window until `expireOverdue()` runs; `recall` is refused immediately.
- Average service time is a floor: `(1+1+2)/3 = 1`.
- Randomised run: after every step every position equals its index in the snapshot, and an all-types counter always receives the head of the snapshot.
- 16 threads issuing, 4 counters calling, serving and cancelling concurrently: no token called twice, `issued = served + cancelled + waiting`.

## Likely interviewer questions
- Define "passed over" in one sentence. Why only later-issued tokens? Why count tokens that a counter could not have served anyway?
- Why is promotion a property of the ordering and not an eager reshuffle of a structure? What does `positionInQueue` cost?
- How do you keep `queueSnapshot`, `positionInQueue` and `callNext` from disagreeing?
- What are the legal transitions of a token? Which ones are one-way? Why is a second recall an error rather than a no-op?
- What breaks if two counters call at the same time? What would you lock?
- Why does the clock have to be injected, and what would `expireOverdue` look like as a background task?

## Invariants
- Every token is in exactly one state; transitions are exactly: WAITING→CALLED/CANCELLED, CALLED→SERVED/NO_SHOW, NO_SHOW→WAITING (once)/EXPIRED.
- One serving order serves `callNext`, `positionInQueue` and `queueSnapshot`.
- Token numbers are unique and gap-free until `resetDay()`.
- A token is called at most once per WAITING period, and by exactly one counter.
- Statistics are derived from token states, except `issued`.

## Common mistakes
- Counting a pass-over only against tokens the calling counter could have served, or only for the head of the queue.
- Counting tokens issued earlier (smaller number) as passing over later ones.
- Promoting by priority instead of age, or letting a promoted token lose its place when a higher-priority one arrives.
- Consuming or reordering tokens a counter skipped.
- Recall as plain re-insert with the old number (not "front of the class"), or forgetting the recall-once flag, or consuming the flag on a refused recall.
- `>=` instead of `>` on the grace/recall boundaries; stamping an expiry-created no-show with the old call time.
- Averaging over all called tokens, or with floating point, or forgetting the 300000 default.
- `stats()` counting CALLED tokens as waiting, or EXPIRED outside the noShow column.
- A non-atomic `n = ++seq` / format pair, or a check-then-act race between `callNext` and `cancel`.

## Over-engineering traps
- A strategy hierarchy per customer type, an event bus, or a scheduler thread for expiry before the spec asks for it.
- A different data structure per counter (a queue per counter) that cannot answer a global position.
- A State-pattern class per status with no behaviour other than allowed transitions.

## Design and scaling discussion
- Patterns an interviewer hopes to hear: a priority queue with aging (comparator over a derived rank), a token life-cycle state machine, dependency injection of the clock, a single aggregate (the system) owning all token state behind one lock.
- With a few hundred waiting tokens sort-on-demand is fine; at scale use an ordered set re-keyed on change, or separate per-class queues plus a promoted list (promotion happens only at a call, so it can be maintained incrementally: after each call, scan lower-numbered waiting tokens).
- Complexity: a call is O(n) pass-over update + O(n log n) ordering in the naive approach; a candidate can reach O(log n) amortised with per-class queues and a promoted list keyed by number.
- Concurrency: one lock is correct and enough; finer locking buys little because every call touches the whole waiting set.
- Production: expiry becomes a timer; positions are approximations the display board refreshes; per-counter queues are an alternative model that loses a global position.

# Interviewer notes — 74 Concurrent LRU Cache (Read-Write Lock)

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: race, deadlock, starvation, memory visibility, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Capacity 1: every `put` of a new key evicts the previous one.
- `put` of an existing key at full capacity must not evict anyone.
- `remove` of the most / least recently used entry (list head and tail).
- A `get` racing with the eviction of the same key.
- Many threads `get` the same key at once.

## Likely interviewer questions
- Which operations take the read lock and which the write lock? Why can't `get` use the read lock?
- Why can a read lock not be upgraded to a write lock? What do you do instead and what must you re-check afterwards?
- Is a read-write lock even faster than a plain mutex here? When is it slower?
- Reader vs writer preference: can writers starve? Can readers?
- How do you keep `size()` from ever being observed above capacity?

## Invariants
- `map.size() == list.size() <= capacity` whenever no lock is held.
- Each key appears once in the list and once in the map, pointing at the same node.
- The list order is the usage order.

## Common mistakes
- `get` under the read lock (corrupts the list, shows up as lost entries or an infinite loop).
- Upgrade attempt: read lock held, then asking for the write lock -> self-deadlock.
- Evicting after releasing the lock, so `size()` exceeds capacity transiently.
- Updating the map but not the list on replace.
- Returning an internal node that other threads may still mutate.

## Over-engineering traps
- Generic eviction-policy plug-ins before the LRU works.
- Reaching for `ConcurrentHashMap` + a separate lock-free list: the two structures cannot be updated atomically together.

## Concurrency discussion
- Read-write lock helps only if reads dominate *and* reads are truly read-only. LRU `get` is a write in disguise - the honest answer may be one mutex or striping.
- Approximate LRU (CLOCK, sampled, batched access recording) trades exactness for scalability.

## Trade-off questions to ask when the candidate is done
- Why a read-write lock and not a plain mutex? Show the workload in which each wins.
- What happens under high contention on one hot key? What would you shard?
- How would you test it? (invariant check in quiet moments, consistency of size vs contents, hot-key survival)
- How would you make `get` lock-free?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`mvn test -Phidden`. They only use the same public contract.

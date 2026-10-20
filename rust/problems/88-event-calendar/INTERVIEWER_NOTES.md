# Interviewer notes — 88 Shared Event Calendar

> **Candidates: do not read this file during practice.** It contains hidden edge cases and the discussion guide.

## How to run the session
1. Let the candidate design first; do not hand out the hints. When they share code, say **correct or incorrect** first, and for incorrect name the bug *type*: wrong behaviour / missed requirement, broken invariant, edge case, complexity problem, or wrong design.
2. If they are stuck, give only a very small nudge (`HINTS.md` Level 1 wording). Never give the full solution unless asked.
3. When the tests pass, ask the trade-off questions below, then open `FOLLOWUPS.md`.

## Hidden edge cases (see `interviewer/`)
- Touching events (end == next start) for the same person and across organizer/attendee roles.
- An agenda window that starts inside an event, ends exactly at an event's start, or ends exactly at its end.
- Conflict for the last attendee in the list: the first attendees must not be left with a phantom event.
- Recurring series where only the last occurrence conflicts; `everyMinutes == end - start` (back-to-back occurrences).
- Decline followed by a new booking in the freed slot; cancel followed by a new booking by an attendee.
- Cross-inviting groups (A invites B while B invites A) from many threads: deadlock.
- `commonFreeSlots` where one user's busy block starts before `from` or runs past `to`; a duration longer than the whole window.

## Likely interviewer questions
- How do you decide the conflict for the whole group before writing anything? What is the unit of atomicity?
- Why do two crossing bookings deadlock if locks are taken in the order the participants appear? How did you prevent it?
- How do you find an overlap in O(log n) with a sorted structure and why is checking only two neighbours enough?
- What does the agenda do for an event that begins before the window?
- How do you merge busy intervals of several people for the free-slot query? Complexity?

## Invariants
- For every user the stored events are pairwise non-overlapping.
- An event is present in the calendar of exactly its current participants (organizer plus attendees who did not decline) and in the id index.
- A failed operation leaves no trace (no event, no partial series).
- An event with start >= end never exists.

## Common mistakes
- Checking conflicts first and inserting later without holding the locks in between (check-then-act): two crossing meetings both succeed.
- Locking participants in call order -> deadlock for crossing groups.
- Using `<=` in the overlap test, which makes touching events conflict.
- Creating the first occurrences of a recurring series and rolling back by hand when a later one fails, leaving a window where others see them.
- Forgetting that the organizer is also a participant when checking conflicts and when cancelling.
- Agenda that filters on `start >= from` and so misses an event already in progress.

## Over-engineering traps
- A full iCalendar RRULE engine, time zones and DST before the plain version works.
- An interval tree class hierarchy when a sorted map and two neighbour lookups suffice.
- One lock per event plus a global lock plus a lock per user: pick one clear lock story.

## Design and scaling discussion
- Lock granularity: one global lock is trivially correct and fine for a single node; per-user locks in a canonical order scale with disjoint groups; a lock-free design would need a per-user version check and retry.
- Storage: one row per (user, event) with an index on (user, start) makes the overlap check a range query; the multi-participant write is a transaction. In a sharded store it becomes a distributed transaction.
- Recurring series: expand occurrences for a bounded horizon (simple, queryable) versus store a rule and expand lazily (compact, but conflict checks and edits get hard).
- Free-slot search: merge of sorted busy lists is O(total events); for many users, a sweep line or a bitmap per day (e.g. one bit per 5 minutes) is faster.

## Trade-off questions to ask when the candidate is done
- Pessimistic per-user locking versus optimistic versioning with retry for hot calendars (e.g. a CEO's).
- Eager rejection on any conflict versus allowing double booking with a warning, as real calendars do.
- Expanding recurrences up front versus storing a rule; what changes for cancellation of a single occurrence?
- How would you test the deadlock freedom and the atomicity, beyond the provided tests?

## Hidden scenarios

Extra tests that are intentionally not in the default test run live in `interviewer/` and run with
`cargo test --features hidden`. They only use the same public contract.

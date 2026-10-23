# 88 · Shared Event Calendar (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 60 min | Level 2 — contract and tests provided; the domain model, the algorithms and the class design are yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process,
> `HINTS.md` if you are stuck (three levels, smallest nudge first) and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building the scheduling core of a team calendar. People organise meetings and invite colleagues; nobody may ever be double-booked. A meeting either lands in every participant's calendar or in nobody's. Organisers can also create a recurring series (for example a daily stand-up) which is created as a whole or not at all. Users view their agenda for a time window and ask "when are all of us free for at least 30 minutes?". Many people use the service at the same time.

## Glossary

- **Organizer**: the user who creates the event; only they can cancel it.
- **Attendee**: an invited user; they can decline (leave) but not cancel.
- **Participant**: the organizer plus all attendees.
- **Overlap**: two events overlap when `a.start < b.end && b.start < a.end`. Events that only touch (one ends exactly when the next starts) do not overlap.
- **Occurrence**: one event of a recurring series.
- **Free slot**: a time window in which every requested user has no event.

## Requirements

1. Times are minutes (a long / `u64`). An event occupies the half-open range `[start, end)`: events that merely touch (one ends when the next starts) do not conflict.
2. `schedule(organizer, title, start, end, attendees)` creates an event for the organizer and every attendee and returns its id. Users exist implicitly by being named.
3. Invalid input is refused: `end <= start`, a blank title, duplicate people, or the organizer listed among the attendees.
4. If ANY participant already has an overlapping event, the call fails and **nothing** is created for anybody.
5. `scheduleRecurring(..., everyMinutes, count)` creates `count >= 1` occurrences, each `everyMinutes` after the previous one (`everyMinutes >= end - start`). It is all-or-nothing across all occurrences and all participants and returns the ids in time order.
6. `cancel(user, eventId)`: only the organizer may cancel; the event disappears for everybody. `decline(user, eventId)`: an attendee leaves, the event stays for the others; the organizer cannot decline.
7. `agenda(user, from, to)` returns the user's events that overlap `[from, to)`, sorted by start then title, each formatted `start-end:title`.
8. `commonFreeSlots(users, from, to, duration)` returns the maximal windows inside `[from, to)` in which ALL the given users are free for at least `duration` minutes, sorted, formatted `start-end`.

## Constraints

- The calendar is used from many threads at once. Every operation must be linearizable: a multi-person booking takes effect at a single instant or not at all.
- Two meetings that cross-invite the same people (A organises with B, B organises with A) must not deadlock and exactly one may win a contested slot.
- Nobody may ever be observed double-booked, and every accepted meeting appears in exactly the calendars of its participants.
- A year of daily recurrences (365 occurrences) and groups of 30 people must work within a normal test run; the hidden tests run 30 threads that each invite the other 29.
- Complexity: do not scan every event of a user for every new booking if a sorted structure answers in O(log n).

## Examples

```
Conflicts and touching
schedule(ann, "standup", 600, 630, [bob])      -> id1
schedule(bob, "other",   620, 700, [])         -> conflict error (bob busy 620-630); nothing is created
schedule(bob, "other",   630, 700, [])         -> ok (touching is fine)
agenda(bob, 0, 2000)  -> ["600-630:standup", "630-700:other"]

All-or-nothing across people
bob is busy 100-200; schedule(ann, "meeting", 150, 250, [bob, cy]) -> conflict error
agenda(ann) and agenda(cy) stay empty.

Agenda window and ordering
ann has "a" 100-200 and "b" 200-300:
agenda(ann, 150, 200) -> ["100-200:a"]      (a starts before the window but overlaps it; b starts at the end, excluded)
agenda(ann, 200, 250) -> ["200-300:b"]
agenda(ann, 300, 400) -> []
Titles may contain ':' or '-': a "1:1" at 540-570 is listed as "540-570:1:1". Same start -> ordered by title.

Cancel / decline
id = schedule(ann, "sync", 100, 200, [bob, cy])
decline(bob, id)  -> bob's agenda empty, cy's still has "100-200:sync"; bob may now book 100-200 again
decline(ann, id)  -> permission error (organizer)       cancel(bob, id) -> permission error (not organizer)
cancel(ann, "no-such-id") -> unknown-event error        cancel(ann, id) -> removed for everybody

Recurring, all-or-nothing
bob has a blocker overlapping the 3rd day.
scheduleRecurring(ann, "daily", 10000, 10030, [bob], 1440, 5) -> conflict error, ann's agenda stays empty
scheduleRecurring(ann, "daily", 10000, 10030, [cy],  1440, 5) -> 5 distinct ids in time order,
   occurrences 10000-10030, 11440-11470, 12880-12910, ...

Common free slots
ann busy 60-120; bob busy 100-180 and 240-260:
commonFreeSlots([ann, bob], 0, 300, 30)  -> ["0-60", "180-240", "260-300"]
commonFreeSlots([ann, bob], 0, 300, 60)  -> ["0-60", "180-240"]
commonFreeSlots([ann],      0, 300, 100) -> ["120-300"]
commonFreeSlots([ann, bob], 0, 300, 200) -> []
```

## Edge cases

- Invalid input (`end <= start`, blank/whitespace title, duplicate attendee, organizer also listed as attendee, recurring with `count < 1` or `everyMinutes < end - start`) is refused and creates nothing.
- Users are never registered: a user with no events has an empty agenda and is free everywhere.
- Event ids are unique non-blank strings; the format is yours, but each occurrence of a series has its own id.
- Two meetings that cross-invite the same people for the same slot (ann organises with bob, bob organises with ann): exactly one succeeds, the other gets the conflict error.
- `cancel` checks that the event exists first, then that the caller is the organizer.

## Behavioral contract

The tests rely on exactly the following. Anything not stated is yours to decide (and to state as an assumption).

- Java: `CalendarApi` with exceptions: `IllegalArgumentException` (invalid input, unknown event id), `IllegalStateException` (conflict), `SecurityException` (non-organizer cancel, organizer decline). Rust: `Calendar` trait, `u64` times, `CalError::{InvalidInput, Conflict, UnknownEvent, PermissionDenied}`, `schedule_recurring`, `common_free_slots`.
- Overlap means `a.start < b.end && b.start < a.end`. Touching events never conflict.
- `agenda` includes events that start before `from` but still overlap the window, and excludes events that start at or after `to` (`agenda(ann,150,200)` with events 100-200 and 200-300 returns only the first).
- `agenda` ordering is start ascending, ties by title; the string is `start-end:title` and the title may itself contain `:` or `-`.
- `commonFreeSlots` windows are clipped to `[from, to)`, merged (adjacent busy periods of different users form one busy block) and only windows of length `>= duration` are returned. With a single user it is just that user's gaps.
- A conflicting `schedule` or `scheduleRecurring` leaves every agenda unchanged. A failed recurring series creates none of its earlier occurrences.
- Cancelling frees the slot for every participant immediately; declining frees it for the decliner only. After a decline the person may book the slot again.
- Event ids are unique non-blank strings; their format is yours. `cancel` of an unknown id reports unknown event (checked before the permission check).

## Minimum API the tests expect (Java)

The tests drive your system through the thin contract below (`src/main/java/contract`). It is **not** your domain model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in `solution/Solution.java`.

```java
package contract;

import java.util.List;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Times are minutes (long) since some epoch; an event occupies the half-open range [start, end), so two events
 * that merely touch (one ends when the other starts) do not conflict. Users exist implicitly by being named.
 */
public interface CalendarApi {

    /**
     * Schedules an event for the organizer and the attendees and returns its id. IllegalArgumentException for
     * end <= start, a blank title, or duplicate people (the organizer may not be listed as an attendee).
     * IllegalStateException if ANY of the people already has an overlapping event; nothing is created then.
     */
    String schedule(String organizer, String title, long start, long end, List<String> attendees);

    /**
     * Schedules {@code count} occurrences (count >= 1) starting at start, each repeating {@code everyMinutes} later
     * (everyMinutes >= end - start so occurrences do not overlap). All-or-nothing: if any occurrence conflicts for
     * anybody, IllegalStateException and nothing is created. Returns the ids in time order.
     */
    List<String> scheduleRecurring(
            String organizer, String title, long start, long end, List<String> attendees, long everyMinutes, int count);

    /** Only the organizer may cancel; the event disappears for everybody. IllegalArgumentException for an unknown id, SecurityException for others. */
    void cancel(String user, String eventId);

    /** An attendee leaves the event; it stays for the others. The organizer cannot decline (SecurityException). */
    void decline(String user, String eventId);

    /** The user's events overlapping [from, to), sorted by start then title, formatted "start-end:title". */
    List<String> agenda(String user, long from, long to);

    /**
     * Windows inside [from, to) where ALL the users are free for at least {@code duration} minutes, merged and sorted,
     * formatted "start-end".
     */
    List<String> commonFreeSlots(List<String> users, long from, long to, long duration);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    public static CalendarApi create()
}
```

## What the tests check

These tests are black-box: they only call the contract above, so any class design behind it works. Time-dependent behaviour uses an injectable fake clock, never real sleeping. More edge-case scenarios exist in `interviewer/` (see `INTERVIEWER_NOTES.md`).

- Agenda sorting by start, window overlap semantics, touching events allowed, and an empty agenda for a user who was never named.
- A conflict for any one participant (here the attendee `bob`) refuses the event and creates it for nobody (`ann` and `cy` stay empty).
- Invalid events: zero and negative length, blank title, duplicate attendee, organizer as attendee.
- Cancel by organizer frees everybody; cancel by an attendee fails with the permission error; unknown id fails.
- Decline: the attendee's slot is free again, others keep the event, the organizer cannot decline.
- Recurring series with a blocker on the 3rd occurrence: whole series refused and `ann` has nothing; then a clean series of 5 creates 5 distinct ids and agenda entries spaced by 1,440.
- Common free slots for two users with several thresholds and for a single user.
- Concurrency: two crossing meetings with the same slot (exactly one wins, 50 rounds); 12 threads randomly booking pairs among 6 people (no overlaps, entries == 2 x accepted); a ring of 6 people booking `p[t]` with `p[t+1]` at the same time (between 1 and 3 winners, repeated 100x).
- More edge cases (a year of recurrences, 30 users all inviting each other for deadlock freedom) are in `interviewer/`.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/88-event-calendar test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green. Test code: `src/test/java/playground/BehaviorTest.java`. Shared helpers (`FakeClock`, `Candidate`): `java/testkit`.

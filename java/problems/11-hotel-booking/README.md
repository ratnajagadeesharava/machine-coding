# 11 · Hotel Booking (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building the reservation engine of a hotel. Guests do not pick a specific room: they ask for a room **type**
(for example `DOUBLE`) for a range of nights, and the system picks a concrete free room for them. A guest can either
book straight away, or place a short **hold** (15 minutes) while they pay and then confirm it. The one thing that
must never happen is the same room being promised to two guests for the same night, even when many requests arrive at
once.

### Terms

- **Room** - has an id (`R1`), a type (`SINGLE`, `DOUBLE`, `SUITE`, ... any string) and a nightly rate.
- **Night / day** - days are plain integers (`1`, `2`, ...). A stay `[checkIn, checkOut)` occupies the nights `checkIn, checkIn+1, ..., checkOut-1`; the checkout day itself is *not* a night of the stay.
- **Booking** - a confirmed stay; it does not expire.
- **Hold** - a temporary booking that blocks the room for 15 minutes unless confirmed. The id of a hold is the id of the booking it becomes on `confirm`.
- **Free on a night** - no booking and no unexpired hold covers that night for that room.

## Requirements

1. The hotel has **rooms**, each with an id, a **type** (e.g. `SINGLE`, `DOUBLE`, `SUITE`) and a nightly rate. Guests book a **type**, not a specific room. Rooms are added with `addRoom` before use.
2. A stay covers nights `[checkIn, checkOut)` (days are integers): checking out on day `d` frees the room for a guest checking in on day `d`.
3. `book` confirms a stay immediately. The system assigns the free room of that type with the **lexicographically smallest id** that is free for **all** the requested nights (plain string comparison: `R1 < R2 < R3`).
4. `hold` reserves a room for the same kind of request but only for **15 minutes**; `confirm` turns an unexpired hold into a booking. Expired holds release the room automatically (nobody calls cleanup), and an expired hold can no longer be confirmed.
5. `cancel` releases a booking or an unexpired hold; the room is free again for those nights.
6. Availability queries: how many rooms of a type are free on a given night (neither booked nor held).
7. The price of a stay is `nights × nightly rate` of the assigned room, where `nights = checkOut - checkIn`.
8. Requests with `checkOut <= checkIn`, an unknown type, or no room free for the whole range are refused (empty result).
9. No room may ever be booked or held twice for an overlapping night - even under concurrent requests.

## Constraints

- Hundreds of rooms, stays up to a year, tens of thousands of bookings. Availability must not require scanning every booking when asked about a single night (be ready to discuss a better structure).
- Time comes from the `Clock` given to the factory.
- Operations must be safe to call from many threads.

## Examples

Hotel: `R1`, `R2` are `DOUBLE` at 100/night; `S1` is a `SUITE` at 300/night.

```
book(alice, DOUBLE, 1, 4) -> B1, room R1       availableRooms(DOUBLE, 1) = 1, (DOUBLE, 3) = 1,
                                               (DOUBLE, 4) = 2 (checkout night is free), (DOUBLE, 0) = 2
book(bob,   DOUBLE, 4, 6) -> B2, room R1       back-to-back: checkout day 4 == check-in day 4
book(carol, DOUBLE, 2, 5) -> B3, room R2       R1 is busy on nights 2 and 3
book(dave,  DOUBLE, 3, 4) -> empty             R1 busy (alice, night 3), R2 busy (carol, night 3)
totalPrice(B3)            -> 300               3 nights x 100
```

Holds (suite `S1`, `now` = T):

```
hold(alice, SUITE, 1, 3)  -> H1
book(bob, SUITE, 2, 4)    -> empty             S1 is held on night 2
(T + 15 min - 1 ms)       confirm(H1) -> true  (still valid; H1 is now a booking and never expires)
alternatively, at T + 15 min exactly: the hold has expired, availableRooms(SUITE, 1) = 1,
              confirm(H1) -> false, and book(bob, SUITE, 1, 3) succeeds
```

Cancel: `cancel(B1)` -> `true`, `roomOf(B1)` -> empty, `cancel(B1)` again -> `false`, `cancel("nope")` -> `false`.

Invalid requests (all return empty): `book(a, DOUBLE, 3, 3)` (zero nights), `book(a, DOUBLE, 5, 2)` (checkout before check-in),
`book(a, PENTHOUSE, 1, 2)` (unknown type; `availableRooms(PENTHOUSE, 1)` is `0`).

## Edge cases

- Smallest-id assignment is by *free for the whole range*: with `R1` busy on night 1 only, a stay `[1, 2)` goes to `R2`, while a later stay on other nights can still go to `R1`.
- Different room types never interfere with each other.
- `confirm` is `true` at most once per hold; confirming an already confirmed hold, a cancelled hold or an unknown id is `false`.
- `roomOf` and `totalPrice` of an unknown or cancelled id are empty.
- Many threads booking the same single suite for the same nights: exactly one succeeds. Many threads booking the same dates across `k` rooms: exactly `k` succeed, each in a different room.

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `book`/`hold` return an id, or empty when refused. The id of a hold stays the id of the booking after `confirm`.
- `roomOf(id)` is the assigned room id; empty for unknown/cancelled ids. `totalPrice(id)` is `nights × rate`.
- A hold is valid while `now < heldAt + 15 min`. `confirm` returns `true` only for an active (unexpired, unconfirmed) hold.
- `cancel(id)` returns `true` if a booking or an unexpired hold was released; `false` for unknown, expired, or already-cancelled ids.
- `availableRooms(type, day)` counts rooms of that type with no booking and no unexpired hold covering **night** `day`.

## Test entry point (Java)

The tests drive your system through the thin, primitive-typed contract below. It is **not** your domain
model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in
`solution/Solution.java`.

```java
package contract;

import java.util.Optional;

/** Thin test entry point. Your domain model lives behind it. Must be thread-safe. */
public interface HotelBookingApi {

    void addRoom(String roomId, String type, long nightlyRate);

    /** Confirmed stay for nights [checkInDay, checkOutDay). Returns the booking id, or empty if refused. */
    Optional<String> book(String guest, String roomType, int checkInDay, int checkOutDay);

    /** Same as book, but the room is only held for 15 minutes. */
    Optional<String> hold(String guest, String roomType, int checkInDay, int checkOutDay);

    /** True only for an active (unexpired, unconfirmed) hold. */
    boolean confirm(String holdId);

    /** Releases a booking or an unexpired hold. */
    boolean cancel(String id);

    /** Rooms of the type free (no booking, no unexpired hold) on the given night. */
    int availableRooms(String roomType, int day);

    Optional<String> roomOf(String id);

    Optional<Long> totalPrice(String id);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    HotelBookingApi create(Clock clock)
}
```

## Language notes

`Clock` is `java.time.Clock`. All contract methods are called from multiple threads in the concurrency tests.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/11-hotel-booking test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

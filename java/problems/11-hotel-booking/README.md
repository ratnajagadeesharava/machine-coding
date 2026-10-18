# 11 · Hotel Booking (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Hard | 90 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

Design the reservation engine of a hotel: guests book a room *type* for a range of nights, the system assigns a
concrete room, supports temporary holds while the guest pays, and guarantees a room is never promised twice for the same night.

## Requirements

1. The hotel has **rooms**, each with an id, a **type** (e.g. `SINGLE`, `DOUBLE`, `SUITE`) and a nightly rate. Guests book a **type**, not a specific room.
2. A stay covers nights `[checkIn, checkOut)` (days are integers): checking out on day `d` frees the room for a guest checking in on day `d`.
3. `book` confirms a stay immediately. The system assigns the free room of that type with the **lexicographically smallest id** that is free for **all** the requested nights.
4. `hold` reserves a room for the same kind of request but only for **15 minutes**; `confirm` turns an unexpired hold into a booking. Expired holds release the room automatically (nobody calls cleanup).
5. `cancel` releases a booking or a hold.
6. Availability queries: how many rooms of a type are free on a given night (neither booked nor held).
7. The price of a stay is `nights × nightly rate` of the assigned room.
8. Requests with `checkOut <= checkIn`, an unknown type, or no room free for the whole range are refused.
9. No room may ever be booked or held twice for an overlapping night — even under concurrent requests.

## Constraints

- Hundreds of rooms, stays up to a year, tens of thousands of bookings. Availability must not require scanning every booking when asked about a single night (be ready to discuss a better structure).
- Time comes from the `Clock` given to the factory.
- Operations must be safe to call from many threads.

## Examples

```
rooms: R1, R2 (DOUBLE, 100/night)
book(alice, DOUBLE, 1, 4) → B1 in R1          availableRooms(DOUBLE, 2) = 1
book(bob,   DOUBLE, 4, 6) → B2 in R1          (checkout day 4 == checkin day 4)
book(carol, DOUBLE, 2, 5) → B3 in R2
book(dave,  DOUBLE, 3, 4) → refused           (R1 busy, R2 busy)
```

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

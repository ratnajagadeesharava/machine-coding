# 13 · Bus / Train Seat Booking (Java)

| Difficulty | Duration | Scaffolding |
|---|---|---|
| Medium-Hard | 90 min | Level 1 — almost empty. Only a thin, primitive-typed test entry point is provided. Every domain class is yours. |

> Read the requirements, then **design before you code**: which concepts, which owners, which API,
> which state, which abstractions (if any). Nothing here tells you. See `INTERVIEW.md` for the process and
> `FOLLOWUPS.md` (open one at a time, after the previous version passes).

## Problem

You are building the booking engine for an intercity bus or train. The vehicle runs through an ordered list of stops
(for example `A, B, C, D`) and has a fixed number of numbered seats. A passenger buys a seat **from one stop to a
later stop**. Because a seat that is taken from `A` to `C` is empty again after `C`, the *same physical seat can be
sold several times along the route* to passengers whose journeys do not overlap. The engine must find a seat that is
free for the **whole** journey, never sell a seat twice for overlapping stretches, and keep that promise when many
passengers book at once.

### Terms

- **Trip** - one run of the vehicle: an id, an ordered list of distinct stop names, and `N` seats numbered `1..N`.
- **Segment** - the stretch between two consecutive stops. With stops `[A, B, C, D]` the segments are `A-B`, `B-C`, `C-D`.
- **Journey** `from -> to` - uses every segment from `from` up to (not including) `to`. `A -> C` uses `A-B` and `B-C`.
- **Free for a journey** - the seat is not booked on *any* segment of that journey.
- **Booking** - a passenger's claim on one seat for one journey; it has an opaque string id.

## Requirements

1. A **trip** has an id, an ordered list of stops and a number of seats numbered `1..N`.
2. A **booking** is for a passenger from a boarding stop to a destination stop that comes **later** in the route. It occupies its seat on every segment between consecutive stops in `[from, to)`.
3. `book` automatically assigns the **lowest-numbered seat** that is free for the whole journey. `bookSeat` requests a specific seat; it is refused if that seat is busy on any segment of the journey.
4. Two bookings may share a seat only if their journeys do not overlap (one may end where the next one starts, e.g. `A -> C` and `C -> D`).
5. A booking can be **cancelled**, which frees its seat for exactly the segments it occupied (other bookings of the same seat are untouched).
6. Availability: how many seats are free for the whole journey between two stops.
7. Invalid requests (unknown trip or stop, `from` not before `to`, seat out of range) are refused without side effects.
8. The same seat must never be sold twice for an overlapping journey, even when many passengers book concurrently.

## Constraints

- Up to 200 stops and 500 seats per trip; many trips.
- Thread-safe. Booking contention on a popular trip must not allow double-selling.
- All checks and the assignment of a seat must be atomic for a booking.

## Examples

Trip `T`: stops `[A, B, C, D]`, 2 seats.

```
book(T, A, C, "p1")  -> id P1, seat 1
book(T, C, D, "p2")  -> id P2, seat 1      journeys A-C and C-D do not overlap, so seat 1 is reused
book(T, B, D, "p3")  -> id P3, seat 2      seat 1 is busy on B-C (p1); seat 2 is free on B-C and C-D
availableSeats(T, B, C) = 0                seat 1 busy (p1), seat 2 busy (p3)
book(T, B, C, "p4")  -> empty              no seat free on B-C
cancel(P1)           -> true               frees seat 1 on A-B and B-C only
availableSeats(T, A, C) = 1                seat 1 (seat 2 is busy on B-C because of p3)
```

After the cancel, `availableSeats(T, C, D) = 0`: seat 1 is held by `p2` and seat 2 by `p3`.

Specific seats (fresh 2-seat trip): `bookSeat(T, 2, A, B, "p1")` -> seat 2; again `bookSeat(T, 2, A, B, ...)` -> empty (busy);
`bookSeat(T, 2, B, C, ...)` -> present (no overlap); a following `book(T, A, B, ...)` gets seat 1 (seat 2 is taken on `A-B`).

Lowest seat after a cancellation: seat 1 and seat 2 both booked `A -> D`; cancel the seat-1 booking; `book(T, B, C, ...)` gets seat 1.

## Edge cases

- Invalid journeys all return empty with no side effect: `from == to`, `from` after `to`, an unknown stop name, an unknown trip.
- Seat numbers `0`, negative numbers and `N+1` are refused by `bookSeat`.
- `availableSeats` is `0` for an invalid journey (unknown trip or stop, `from` not before `to`), not an error.
- `seatOf` of an unknown or cancelled booking is empty; `cancel` is `true` only the first time.
- Trips are independent: bookings on one trip never change another trip's availability.
- 100 passengers racing for a 40-seat `A -> B` trip: exactly 40 succeed, each with a different seat.

## Behavioral contract

The tests rely on exactly the following behavior. Anything not stated is yours to decide (and to state as an assumption).

- `book`/`bookSeat` return a booking id, or empty if refused. `seatOf(id)` is the assigned seat; empty for unknown or cancelled bookings.
- `availableSeats(trip, from, to)` = seats free on **every** segment of the journey; 0 for invalid journeys.
- `cancel(id)` returns `true` the first time, `false` for unknown or already cancelled ids.
- Stops are identified by name; `from` must appear strictly before `to` in the trip's stop list.

## Test entry point (Java)

The tests drive your system through the thin, primitive-typed contract below. It is **not** your domain
model — it is the doorway. Create whatever classes you like in `src/main/java` and wire them up in
`solution/Solution.java`.

```java
package contract;

import java.util.List;
import java.util.Optional;
import java.util.OptionalInt;

/** Thin test entry point. Your domain model lives behind it. Must be thread-safe. */
public interface SeatBookingApi {

    void addTrip(String tripId, List<String> stops, int seats);

    /** Lowest-numbered seat free for the whole journey. Returns the booking id, or empty if refused. */
    Optional<String> book(String tripId, String from, String to, String passenger);

    /** A specific seat (1..N). */
    Optional<String> bookSeat(String tripId, int seat, String from, String to, String passenger);

    OptionalInt seatOf(String bookingId);

    boolean cancel(String bookingId);

    /** Seats free on every segment of the journey; 0 for invalid journeys. */
    int availableSeats(String tripId, String from, String to);
}
```

Your wiring point (`src/main/java/solution/Solution.java`):

```java
public final class Solution {
    SeatBookingApi create()
}
```

## Language notes

All methods are called from multiple threads. Booking ids are opaque strings.

## Running

```bash
mvn test                 # from this directory (or `mvn -pl problems/13-seat-booking test` from java/)
STRICT=1 mvn test       # fail instead of skip while unimplemented
mvn test -Phidden       # interviewer scenarios (see INTERVIEWER_NOTES.md)
```

Tests that need your implementation are **skipped** (not failed) until `create` is wired up, so a fresh checkout builds green.

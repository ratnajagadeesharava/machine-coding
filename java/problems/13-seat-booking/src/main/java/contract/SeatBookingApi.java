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

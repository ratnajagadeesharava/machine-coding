package contract;

import java.util.List;
import java.util.Optional;

/** Thin test entry point. Your domain model lives behind it. Must be thread-safe. */
public interface FlightBookingApi {

    void addFlight(String flightId, String from, String to, long departureMillis,
                   int economySeats, int businessSeats, long economyFare, long businessFare);

    /** Flights with at least one available seat, ordered by departure, economy fare, then id. */
    List<String> search(String from, String to, long notBeforeMillis);

    /** Business seats first, then economy; each in numeric order. */
    List<String> availableSeats(String flightId);

    /** Holds the seat for 10 minutes and returns the reservation id, or empty if refused. */
    Optional<String> reserve(String flightId, String seat, String passenger);

    /** True only if the reservation is active and payment succeeded. */
    boolean confirm(String reservationId, boolean paymentSucceeds);

    /** Refund amount (possibly 0), or empty if unknown/expired/already cancelled. */
    Optional<Long> cancel(String reservationOrBookingId);

    Optional<Long> fareOf(String flightId, String seat);
}

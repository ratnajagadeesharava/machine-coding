package contract;

import java.util.List;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Times of shows are in minutes; "now" is the injected clock's millis divided by 60,000. Seat labels are
 * a row letter plus a 1-based number: "A1", "A2", ..., "C10". Rows are A, B, C ... from the screen backwards;
 * rows A and B are PREMIUM, all other rows are REGULAR.
 */
public interface TicketingApi {

    /** A screen with {@code rows} rows (1..26) of {@code seatsPerRow} seats. IllegalArgumentException if it exists. */
    void addScreen(String theatre, String city, String screen, int rows, int seatsPerRow);

    /**
     * A show on a screen. IllegalArgumentException for an unknown screen, duration <= 0 or prices <= 0 or a duplicate
     * showId; IllegalStateException if it overlaps another show on the same screen (touching is fine).
     */
    void addShow(String showId, String theatre, String screen, String movie, long startMinute, long durationMinutes,
                 long regularPrice, long premiumPrice);

    /** Ids of the shows of {@code movie} in {@code city} that have not started yet, sorted by start then id. */
    List<String> searchShows(String city, String movie);

    /** Seats that can be held right now (not sold, not under a live hold), in row-then-number order. */
    List<String> availableSeats(String showId);

    /**
     * Holds all the seats for the user for 5 minutes, all-or-nothing, and returns the hold id.
     * IllegalArgumentException for an unknown show/seat, no seats or duplicates; IllegalStateException if any seat is
     * sold or held, or the show has started.
     */
    String hold(String showId, String user, List<String> seats);

    /** Gives a hold up early (SecurityException for another user; IllegalStateException if it no longer exists). */
    void release(String holdId, String user);

    /**
     * Turns a live hold into a booking and returns the booking id. SecurityException for another user;
     * IllegalStateException if the hold has expired or was already used.
     */
    String confirm(String holdId, String user);

    /** Sorted seat labels of a booking. */
    List<String> seatsOf(String bookingId);

    /** Sum of the seat prices (premium rows A,B at the premium price, the others at the regular price). */
    long totalOf(String bookingId);

    /**
     * Cancels a booking and frees the seats; returns the refund: 100% of the total if the show starts in 120 minutes
     * or more, 50% (rounded down) if 30..119 minutes away, nothing if closer or already started.
     * SecurityException for another user; IllegalStateException if already cancelled.
     */
    long cancel(String bookingId, String user);
}

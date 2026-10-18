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

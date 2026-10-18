package contract;

import java.util.Optional;
import java.util.OptionalInt;

/** Thin test entry point. Your domain model lives behind it. Must be safe to call from multiple threads. */
public interface ParkingLotApi {

    /** Parks the vehicle. Returns a ticket id, or empty if refused. */
    Optional<String> enter(String vehicleType, String plate);

    /** Releases the spot of the ticket and returns the fee, or empty if the ticket is unknown/already used. */
    Optional<Long> exit(String ticketId);

    /** Number of currently free spots that could accept this vehicle type. */
    int availableSpots(String vehicleType);

    /** 0-based floor of the spot held by an active ticket. */
    OptionalInt floorOf(String ticketId);
}

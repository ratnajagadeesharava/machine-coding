package contract;

import java.util.Optional;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Vocabulary: vehicle types are the strings {@code "TWO_WHEELER"} and {@code "FOUR_WHEELER"}; strategies are the
 * strings {@code "LOWEST_INDEX_SPOT"} and {@code "MOST_FREE_FLOOR"}. A spot is described as {@code "floor:spot"}
 * (for example {@code "1:4"}) where {@code spot} is the global spot index (see the README for the numbering).
 */
public interface ParkingLot {

    /**
     * Parks a vehicle in a free spot of its own type, chosen by the current strategy, and returns the ticket id.
     * Returns empty (and changes nothing) if no compatible spot is free.
     *
     * @throws IllegalArgumentException if the vehicle type is unknown or the registration number is null/blank
     * @throws IllegalStateException    if a vehicle with this registration number is currently parked (checked
     *                                  before looking for a spot, so it also applies when the lot is full)
     */
    Optional<String> park(String vehicleType, String registrationNumber);

    /**
     * Frees the spot held by the ticket. Returns true if the ticket was active; false for an unknown ticket or one
     * that was already used. The freed spot is available to the very next {@code park}.
     *
     * @throws IllegalArgumentException if the ticket id is null/blank
     */
    boolean unpark(String ticketId);

    /**
     * "floor:spot" of the vehicle with this registration number, or empty if it is not currently parked.
     *
     * @throws IllegalArgumentException if the registration number is null/blank
     */
    Optional<String> findByRegistration(String registrationNumber);

    /**
     * "floor:spot" held by an active ticket, or empty for an unknown or already used ticket.
     *
     * @throws IllegalArgumentException if the ticket id is null/blank
     */
    Optional<String> findByTicket(String ticketId);

    /**
     * Number of free spots of this type in the whole lot.
     *
     * @throws IllegalArgumentException if the vehicle type is unknown
     */
    int freeSpots(String vehicleType);

    /**
     * Number of free spots of this type on one floor.
     *
     * @throws IllegalArgumentException if the vehicle type is unknown or the floor does not exist
     */
    int freeSpotsOnFloor(int floor, String vehicleType);

    /**
     * Switches the allocation strategy for all later {@code park} calls; vehicles already parked do not move.
     *
     * @throws IllegalArgumentException if the strategy is unknown
     */
    void setStrategy(String strategy);
}

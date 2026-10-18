package contract;

/** Thin test entry point. Your domain model lives behind it. */
public interface ElevatorApi {

    /** Hall call. direction is "UP" or "DOWN". Returns false for invalid requests. */
    boolean requestPickup(int floor, String direction);

    /** Cabin button inside the given elevator (0-based). Returns false for invalid requests. */
    boolean requestDestination(int elevator, int floor);

    /** Advances the simulation by one tick. */
    void step();

    int floorOf(int elevator);

    /** True only during the step in which the doors were opened. */
    boolean doorsOpen(int elevator);
}

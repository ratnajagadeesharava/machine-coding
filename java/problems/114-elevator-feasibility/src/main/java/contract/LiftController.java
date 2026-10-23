package contract;

import java.util.List;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>A building has floors {@code 0..floors-1}, one lift, a capacity in persons and a limit on intermediate stops.
 * The lift starts idle at floor 0 and only moves when {@link #step()} is called.
 */
public interface LiftController {

    /**
     * Asks to be carried from {@code fromFloor} to {@code toFloor}. Returns {@code true} (accepted: the promise is
     * recorded) or {@code false} (rejected: the state is exactly as before the call).
     *
     * <p>Accepted only if, when the lift follows its sweep (see the README) with the new request added, the load never
     * exceeds the capacity and every rider (the new one and every rider accepted earlier) reaches their destination
     * with at most the configured number of intermediate stops. A request for more persons than the capacity is
     * rejected ({@code false}).
     *
     * @throws IllegalArgumentException for a null/blank {@code requestId}, an id that belongs to a rider that is still
     *     waiting or onboard, a floor outside the building, {@code fromFloor == toFloor}, or {@code persons <= 0}
     */
    boolean request(String requestId, int fromFloor, int toFloor, int persons);

    /**
     * Moves the lift to its next stop, lets the riders destined for that floor off and the riders waiting there (in
     * the sweep's direction) on. Returns the floor of that stop, or {@code -1} (and does nothing) when no accepted
     * rider is waiting or onboard.
     */
    int step();

    /** The floor the lift is at. */
    int currentFloor();

    /** Total persons currently inside the lift. */
    int onboard();

    /**
     * The floors of all upcoming stops, in the order the lift will make them, until every accepted rider is
     * delivered. Empty when idle. Calling {@link #step()} repeatedly returns exactly these floors in this order
     * (as long as nothing else is accepted in between).
     */
    List<Integer> plan();

    /** {@code "WAITING"}, {@code "ONBOARD"}, {@code "DELIVERED"}, or {@code "UNKNOWN"} (never accepted / rejected). */
    String status(String requestId);
}

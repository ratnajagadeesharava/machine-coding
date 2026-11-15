package contract;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 */
public interface OrderedPrinter {

    /**
     * Blocks until it is {@code worker}'s turn, runs {@code action} on the calling thread, then hands the turn to
     * worker {@code (worker + 1) % workers}. The turn is handed over even if the action throws (the exception
     * propagates to the caller). IllegalArgumentException if worker is outside [0, workers).
     */
    void takeTurn(int worker, Runnable action) throws InterruptedException;
}

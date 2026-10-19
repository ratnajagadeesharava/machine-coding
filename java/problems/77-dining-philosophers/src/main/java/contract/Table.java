package contract;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Philosophers sit in a circle, numbered 0..seats-1; there is one fork between every two neighbours, so
 * philosopher p shares a fork with (p + 1) % seats and with (p - 1 + seats) % seats.
 */
public interface Table {

    /**
     * Blocks until philosopher {@code p} holds both of its forks, runs {@code eat} on the calling thread, then
     * puts the forks down (also when {@code eat} throws; the exception propagates).
     * IllegalArgumentException if p is outside [0, seats).
     */
    void dine(int philosopher, Runnable eat) throws InterruptedException;
}

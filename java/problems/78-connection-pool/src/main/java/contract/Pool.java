package contract;

import java.util.Optional;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>A pool of {@code size} reusable "connections", identified by the numbers 0..size-1.
 */
public interface Pool {

    /**
     * Waits up to {@code timeoutMs} for a free connection and returns its id, or empty if none became free in time.
     * {@code timeoutMs <= 0} means do not wait. Throws IllegalStateException if the pool is (or becomes) closed.
     */
    Optional<Integer> acquire(long timeoutMs) throws InterruptedException;

    /**
     * Gives a connection back. IllegalArgumentException if id is outside [0, size);
     * IllegalStateException if that connection is not currently acquired (double release). Allowed after close.
     */
    void release(int id);

    /** Connections that could be acquired right now. */
    int available();

    /** Connections currently acquired. {@code available() + inUse() == size} at all times. */
    int inUse();

    /** Rejects further acquires and wakes every waiting thread (they throw IllegalStateException). Idempotent. */
    void close();
}

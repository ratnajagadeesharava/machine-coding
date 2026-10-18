package contract;

import java.util.Optional;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface BlockingBuffer<T> {

    /** Waits while full. Throws IllegalStateException if the buffer is (or becomes) closed; the item is then not stored. */
    void put(T item) throws InterruptedException;

    /** Waits while empty and open. Empty only when the buffer is closed and drained. */
    Optional<T> take() throws InterruptedException;

    /** Like put but gives up after timeoutMs (false). timeoutMs <= 0 means do not wait. IllegalStateException if closed. */
    boolean offer(T item, long timeoutMs) throws InterruptedException;

    /** Like take but gives up after timeoutMs (empty). timeoutMs <= 0 means do not wait. */
    Optional<T> poll(long timeoutMs) throws InterruptedException;

    /** Number of stored items. */
    int size();

    /** Rejects further puts, wakes all waiters, keeps remaining items for consumers. Idempotent, never blocks. */
    void close();
}

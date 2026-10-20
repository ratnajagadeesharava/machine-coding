package contract;

import java.util.Optional;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Time comes from the clock handed to the factory. A lease granted at time t for d milliseconds is valid while
 * {@code now < t + d}; at {@code now == t + d} it has expired. Nothing blocks: a caller that does not get the lock
 * gets an empty answer and decides what to do.
 */
public interface LockService {

    /**
     * Tries to take the named lock for {@code leaseMs} (> 0, else IllegalArgumentException).
     * <ul>
     *   <li>Free, or the previous lease has expired: granted with a NEW fencing token = (highest token ever issued for
     *       this lock) + 1. Tokens only ever grow, also across different owners and expired leases.</li>
     *   <li>Held (unexpired) by the same owner: the lease is extended to now + leaseMs and the SAME token is returned.</li>
     *   <li>Held (unexpired) by somebody else: empty.</li>
     * </ul>
     */
    Optional<Long> acquire(String lock, String owner, long leaseMs);

    /** Extends the lease to now + leaseMs. True only if this owner and token still hold an unexpired lease. */
    boolean renew(String lock, String owner, long token, long leaseMs);

    /** Gives the lock up. True only if this owner and token still hold an unexpired lease; otherwise nothing changes. */
    boolean release(String lock, String owner, long token);

    /** The owner of the unexpired lease, if any. */
    Optional<String> holder(String lock);

    /** The highest token ever issued for the lock; 0 if it was never taken. */
    long latestToken(String lock);
}

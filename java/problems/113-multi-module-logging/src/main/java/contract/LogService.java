package contract;

import java.util.List;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe: many threads call {@code log} at once.
 *
 * <p>Fixed values (case-sensitive): modules {@code AUTH}, {@code PAYMENTS}, {@code SEARCH}; destinations
 * {@code FILE}, {@code TOOL}; levels {@code DEBUG < INFO < WARN < ERROR}.
 *
 * <p>Line format, used for what a sink receives and for what {@code query} returns:
 * {@code <seq>|<MODULE>|<DESTINATION>|<LEVEL>|<activity>} (for example {@code 7|AUTH|FILE|WARN|bad password}).
 * The activity text is kept verbatim and may itself contain {@code |}.
 */
public interface LogService extends AutoCloseable {

    /**
     * Accepts a log and returns its sequence number once the destination sink has written it successfully.
     * Sequence numbers start at 1 and are gap-free: a log whose sink write fails is NOT given a number, it is not
     * stored, and the next successful log gets the next number. Sinks receive lines in increasing sequence order.
     * Returns {@code -1} when the sink write failed (returned false or threw).
     * Blocks the caller while the shared bounded buffer is full or until the write has been attempted.
     * A log that was acknowledged is immediately visible to {@link #query}.
     *
     * @throws IllegalArgumentException if module, destination or level is not one of the fixed values, or the
     *         activity is null or blank (nothing is consumed or written)
     * @throws IllegalStateException if the service has been closed
     */
    long log(String module, String destination, String level, String activity);

    /**
     * Returns the acknowledged logs matching every non-null filter, as lines, in increasing sequence order.
     * {@code level} is a threshold: logs at that level or above match. {@code fromSeq} and {@code toSeq} are
     * inclusive bounds. The result is a snapshot the caller may modify. Never returns null.
     *
     * @throws IllegalArgumentException for an unknown destination, module or level, or when both bounds are given
     *         and {@code fromSeq > toSeq}
     */
    List<String> query(String destination, String module, String level, Long fromSeq, Long toSeq);

    /**
     * Stops accepting logs, lets everything already accepted finish, and stops any background thread before
     * returning. Safe to call more than once. After it returns, {@code log} throws IllegalStateException while
     * {@code query} keeps working.
     */
    @Override
    void close();
}

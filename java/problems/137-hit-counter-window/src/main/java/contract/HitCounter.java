package contract;

import java.util.List;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 * <p>
 * Time comes from the {@code java.time.Clock} given to {@code Solution.create} (only {@code clock.millis()} is used).
 * Every operation, reads included, takes ONE clock reading and uses {@code now = max(reading, the largest now this
 * counter has used before)}; a clock that moves backwards is therefore never seen going back. A hit is recorded at
 * that {@code now}. A hit recorded at time {@code t} is inside a window of length {@code w} exactly when
 * {@code now - w < t <= now} (a hit that is exactly {@code w} ms old has expired).
 * <p>
 * Blank or null keys are an IllegalArgumentException everywhere. An unknown (but valid) key counts as 0.
 */
public interface HitCounter {

    /** Records one hit for the key at the current time. */
    void hit(String key);

    /** Records {@code count} hits at the current time. IllegalArgumentException if {@code count < 1}. */
    void hit(String key, long count);

    /** Hits of the key inside the configured window. */
    long count(String key);

    /**
     * Hits of the key in {@code (now - durationMs, now]}. IllegalArgumentException unless
     * {@code 0 < durationMs <= current window}; the duration is validated even for unknown keys.
     */
    long countInLast(String key, long durationMs);

    /** Hits across all keys inside the configured window. */
    long totalInWindow();

    /**
     * Changes the window (IllegalArgumentException if {@code newWindowMs < 1}). First every hit that is already outside
     * the OLD window (at the current time) is gone for good; then the new window applies. Shrinking therefore drops
     * the hits that are not inside the new window permanently, and growing never brings back a hit that was already
     * outside the old window.
     */
    void setWindow(long newWindowMs);

    /** The configured window in milliseconds. */
    long window();

    /**
     * Keys with at least one hit inside the window, ordered by windowed count descending then key ascending
     * (natural {@code String} order), at most {@code k} of them. IllegalArgumentException if {@code k <= 0}.
     * The result is a new list the caller may modify.
     */
    List<String> topKeys(int k);

    /**
     * Approximate memory footprint: the number of timestamp/bucket records this counter currently keeps in memory,
     * summed over all keys (a fixed-size array of slots kept for a key counts every slot of it; a key with nothing
     * stored counts 0). Tests only check upper bounds, never an exact value. May drop expired records first.
     */
    long memoryProbe();
}

package contract;

import java.util.Map;
import java.util.OptionalLong;

/** Thin test entry point. Your domain model lives behind it. Must be thread-safe. */
public interface MetricsApi {

    /** False (nothing recorded) if delta <= 0 or the name is blank. */
    boolean increment(String name, long delta, Map<String, String> tags);

    /** Sum over exactly this tag set; 0 if unknown. */
    long counter(String name, Map<String, String> tags);

    /** Sum over all tag sets of the counter name. */
    long counterTotal(String name);

    /** False if the name is blank. */
    boolean observe(String name, long value);

    /** How many values were observed for the histogram. */
    long observationCount(String name);

    /** Nearest-rank percentile, p in 1..100; empty if unknown/empty or p out of range. */
    OptionalLong percentile(String name, int p);

    /** Sum of deltas (any tags) recorded in (now - windowMillis, now]. */
    long countInWindow(String name, long windowMillis);
}

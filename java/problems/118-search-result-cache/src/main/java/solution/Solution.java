package solution;

import contract.*;
import java.time.Clock;
import java.util.*;
import java.util.concurrent.*;
import java.util.function.*;

/**
 * Wiring point between the test contract and YOUR design.
 * Create your own classes anywhere under src/main/java and return them from here.
 */
public final class Solution {
    private Solution() {}

    /**
     * @param capacity maximum number of stored entries (must be &gt; 0)
     * @param ttlMillis lifetime of an entry in milliseconds (must be &gt; 0)
     * @param clock time source for expiry
     * IllegalArgumentException for a null backend or clock, or a non-positive capacity or ttl.
     */
    public static SearchCache create(SearchBackend backend, int capacity, long ttlMillis, Clock clock) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

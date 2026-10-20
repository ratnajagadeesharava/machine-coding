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

    /** All time-dependent behaviour (hold expiry, "not started yet", refunds) must use this clock. */
    public static TicketingApi create(Clock clock) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

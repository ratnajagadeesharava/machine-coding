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
     * @param notifier receives every notification (the tests implement it)
     * @param clock    time source for sample timestamps and evaluation time
     * IllegalArgumentException for a null notifier or clock.
     */
    public static AlertMonitor create(Notifier notifier, Clock clock) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

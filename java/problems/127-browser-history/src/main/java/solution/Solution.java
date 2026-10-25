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
     * @param capacity maximum number of global history entries kept (at least 1)
     * @param clock source of visit timestamps ({@code clock.millis()})
     */
    public static BrowserHistory create(int capacity, Clock clock) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

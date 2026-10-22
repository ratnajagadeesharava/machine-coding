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

    /** The service writes FILE logs to {@code file}, TOOL logs to {@code tool}; the shared buffer holds {@code bufferCapacity} logs. */
    public static LogService create(Sink file, Sink tool, int bufferCapacity) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

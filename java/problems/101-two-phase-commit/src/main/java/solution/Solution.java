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

    /** A coordinator for the given participants. {@code log} may already contain records (restart). */
    public static CoordinatorNode coordinator(String id, List<String> participants, BiConsumer<String, String> send, DurableLog log) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** A participant that reports to {@code coordinatorId}. {@code log} may already contain records (restart). */
    public static ParticipantNode participant(String id, String coordinatorId, BiConsumer<String, String> send, DurableLog log) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

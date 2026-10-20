package solution;

import contract.*;
import java.util.*;
import java.util.function.*;

/**
 * Wiring point between the test contract and YOUR design.
 * Create your own classes anywhere under src/main/java and return them from here.
 */
public final class Solution {
    private Solution() {}

    /** {@code peers} are the OTHER nodes; {@code seed} seeds this node's random election timeouts. */
    public static RaftNode create(String id, List<String> peers, BiConsumer<String, String> send, long seed) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

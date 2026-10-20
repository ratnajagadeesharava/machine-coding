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
     * @param id    this node's name
     * @param peers the OTHER nodes' names (the cluster has peers.size() + 1 nodes)
     * @param send  {@code send.accept(to, payload)} delivers a message to another node (later, maybe never)
     * @param seed  seed for the election-timeout random numbers
     */
    public static ElectionNode create(String id, List<String> peers, BiConsumer<String, String> send, long seed) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

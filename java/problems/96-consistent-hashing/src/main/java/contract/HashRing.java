package contract;

import java.util.List;
import java.util.Set;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe (lookups run while nodes join and leave).
 *
 * <p>The mapping must depend only on the set of nodes (names and weights) and the key - never on the order in which
 * nodes were added or on a random seed - so that two processes holding the same membership agree on every key.
 */
public interface HashRing {

    /** Adds a node with weight 1. IllegalArgumentException if it is already on the ring or the name is blank. */
    void addNode(String node);

    /** A node of weight w owns about w times as many keys as a node of weight 1. Weight >= 1. */
    void addNode(String node, int weight);

    /** Returns false if the node was not on the ring. Its keys move to the remaining nodes. */
    boolean removeNode(String node);

    /** The node responsible for the key. IllegalStateException if the ring is empty. */
    String nodeFor(String key);

    /**
     * Up to {@code n} DISTINCT nodes for replication: first {@code nodeFor(key)}, then the following distinct nodes
     * clockwise on the ring. Fewer than n if the ring has fewer nodes. Empty list for an empty ring.
     */
    List<String> nodesFor(String key, int n);

    Set<String> nodes();
}

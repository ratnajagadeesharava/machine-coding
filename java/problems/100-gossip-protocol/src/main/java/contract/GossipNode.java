package contract;

import java.util.Optional;
import java.util.Set;

/**
 * Thin test entry point. One instance is ONE node; the tests run the nodes inside a deterministic simulation (see
 * {@code testkit.SimNet}): they deliver messages with {@link #onMessage}, advance time with {@link #onTick} (once per
 * simulated millisecond) and read state through the getters. Nodes may only communicate through the {@code send}
 * function they were created with. Single-threaded: no locking needed.
 *
 * <p>Protocol (push-pull gossip with heartbeat failure detection):
 * <ul>
 *   <li><b>Rounds.</b> Every 100 ms (the first round at a random moment within the first 100 ms, drawn from the node's
 *       own seeded random generator) a node increments its own heartbeat counter, picks up to 2 distinct random peers
 *       and sends each of them its whole state. A node that receives such a push merges it and answers with its own
 *       whole state (the answer is not answered again). That is the only traffic: a node never broadcasts to everybody.</li>
 *   <li><b>Data.</b> {@link #put} writes a key locally with version = (highest version this node knows for the key) + 1,
 *       tagged with the writer's id. The winner for a key is the entry with the higher version; equal versions are
 *       decided by the larger writer id. Merging is therefore commutative, associative and idempotent, so every node
 *       ends with the same value once the gossip has spread.</li>
 *   <li><b>Failure detection.</b> Heartbeat counters travel with the state and are merged by maximum. A node counts a
 *       peer as alive as long as the peer's counter has increased within the last 2000 ms of this node's own time; at
 *       the start every peer is given the benefit of the doubt (as if its counter had just increased). A peer that
 *       shows progress again becomes alive again.</li>
 * </ul>
 * The payload format is yours.
 */
public interface GossipNode {

    void onMessage(String from, String payload);

    /** Called once per millisecond with the current simulated time (milliseconds since the start). */
    void onTick(long nowMs);

    /** Writes locally; the value spreads by gossip. */
    void put(String key, String value);

    /** The value this node currently believes wins for the key. */
    Optional<String> get(String key);

    /** This node itself plus every peer it currently believes to be alive. */
    Set<String> aliveMembers();
}

package contract;

import java.util.Optional;

/**
 * Thin test entry point. One instance is ONE node of a cluster; the tests run the nodes inside a deterministic
 * simulation (see {@code testkit.SimNet}): they deliver messages with {@link #onMessage}, advance time with
 * {@link #onTick} (once per simulated millisecond) and read the state through the getters. Nodes may only
 * communicate through the {@code send} function they were created with. Single-threaded: no locking needed.
 *
 * <p>The protocol is the election half of Raft:
 * <ul>
 *   <li>Every node starts as a FOLLOWER in term 0. A follower that has heard nothing from a leader (or granted a vote)
 *       for its election timeout becomes a CANDIDATE: term + 1, votes for itself, asks every peer for its vote.
 *       The election timeout is drawn uniformly from [150, 300] ms, again after every reset, from a random number
 *       generator seeded with the seed given to the factory (so a run is reproducible).</li>
 *   <li>A node grants at most one vote per term (to the first asker, or again to the same candidate) and resets its
 *       election timer when it does. A candidate with votes from a majority of the whole cluster (itself included)
 *       becomes LEADER and sends a heartbeat at once; a leader sends heartbeats to all peers every 50 ms.</li>
 *   <li>Any message carrying a higher term makes the receiver adopt that term and become FOLLOWER (forgetting its vote).
 *       Messages carrying a lower term are answered with the receiver's term and otherwise ignored. A candidate that
 *       receives a heartbeat of its own term becomes FOLLOWER. A heartbeat of the current term (or higher) resets the
 *       election timer and records the sender as the known leader.</li>
 *   <li>Check-quorum: a leader that has not got heartbeat replies from a majority of the cluster (itself included) within
 *       the last 300 ms steps down to FOLLOWER (same term) and starts a fresh election timer.</li>
 *   <li>A cluster of one node elects itself as soon as its election timer fires.</li>
 * </ul>
 * The payload format is yours.
 */
public interface ElectionNode {

    void onMessage(String from, String payload);

    /** Called once per millisecond with the current simulated time (milliseconds since the start). */
    void onTick(long nowMs);

    Role role();

    long term();

    /** The leader this node currently knows in its current term: itself if it is the leader, empty if unknown. */
    Optional<String> leader();
}

package contract;

import java.util.Optional;

/**
 * Thin test entry point. One instance is ONE replica of a fully replicated key-value store (Dynamo style, leaderless,
 * N = cluster size, W = R = majority). The tests run the nodes inside a deterministic simulation (see
 * {@code testkit.SimNet}): they deliver messages with {@link #onMessage}, advance time with {@link #onTick} (once per
 * simulated millisecond), drop / duplicate / delay messages and crash nodes (a crashed node keeps its memory but sees
 * nothing). Nodes may only communicate through the {@code send} function they were created with. Single-threaded.
 *
 * <p>A node that was crashed learns the time again with its first tick after the recovery.
 *
 * <p>Any node can be the coordinator of a request; clients are asynchronous: they start a request with an id and poll
 * {@link #reply}. Rules:
 * <ul>
 *   <li><b>Versions.</b> A put gets the version (t, coordinatorId) where t is the last time passed to {@code onTick}.
 *       The higher version wins, equal t is decided by the larger id. A replica only overwrites an entry with a higher
 *       version and acknowledges every write it receives (also an old one).</li>
 *   <li><b>put.</b> The coordinator writes locally (that is one ack) and sends the write to all others. The reply is OK
 *       as soon as a majority acknowledged. Unacknowledged nodes are re-sent the write every 100 ms until all of them
 *       acknowledged or 1000 ms after the request started, also after the reply was already OK. No majority within
 *       1000 ms: FAILED (the write may still have reached some replicas).</li>
 *   <li><b>get.</b> The coordinator reads locally (one answer) and asks all others, re-asking silent ones every 100 ms.
 *       When a majority answered, the reply is the entry with the highest version among the answers (OK), or NOT_FOUND
 *       if none of them has the key. <b>Read repair:</b> every node that answered with an older entry (or none) gets the
 *       winning entry sent as a write, also if its answer arrives after the reply was decided (until 1000 ms).
 *       No majority within 1000 ms: FAILED.</li>
 *   <li>A request id is used once; starting a second request with a known id does nothing (client retries are safe).
 *       {@code reply} of an unknown id is PENDING.</li>
 * </ul>
 * The payload format is yours.
 */
public interface KvNode {

    void onMessage(String from, String payload);

    /** Called once per millisecond with the current simulated time (milliseconds since the start). */
    void onTick(long nowMs);

    void put(String requestId, String key, String value);

    void get(String requestId, String key);

    Reply reply(String requestId);

    /** This replica's own copy, for inspection (no coordination). */
    Optional<String> local(String key);
}

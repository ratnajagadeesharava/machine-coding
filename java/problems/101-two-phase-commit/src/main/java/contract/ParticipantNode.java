package contract;

import java.util.Optional;
import java.util.Set;

/**
 * A resource manager in two-phase commit: a small key-value store whose writes are applied atomically with the other
 * participants'. Runs inside the deterministic simulation; single-threaded.
 *
 * <p>Protocol:
 * <ul>
 *   <li>PREPARE(tx, writes): if the transaction was already prepared, answer YES again; if it was already decided or
 *       refused, answer consistently (NO for ABORT/refused, YES for COMMIT). Otherwise the participant votes NO if any
 *       key is locked by another prepared transaction, or any value to write equals the text {@code "REJECT"} (a
 *       stand-in for a business rule). On YES it locks the keys and appends a PREPARED record (with the writes) to its
 *       log BEFORE answering: after that it may not unilaterally change its mind.</li>
 *   <li>DECISION(tx, COMMIT|ABORT): COMMIT applies the writes; both release the locks; the decision is logged and
 *       acknowledged (acknowledge duplicates again). A decision for a transaction that was never prepared here is
 *       remembered too, so that a late PREPARE for it is refused instead of locking keys forever.</li>
 *   <li>In-doubt transactions (prepared, no decision) keep their locks and are asked about every 200 ms (INQUIRE), until
 *       the coordinator answers.</li>
 *   <li>Restart: a new instance created with a non-empty log rebuilds its store, locks and in-doubt set from it.</li>
 * </ul>
 */
public interface ParticipantNode {

    void onMessage(String from, String payload);

    void onTick(long nowMs);

    /** The last committed value of the key. */
    Optional<String> get(String key);

    /** True while a prepared (in-doubt) transaction holds the key. */
    boolean isLocked(String key);

    /** Transactions that are prepared and not decided yet. */
    Set<String> inDoubt();
}

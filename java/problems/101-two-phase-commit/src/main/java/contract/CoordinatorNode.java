package contract;

import java.util.Map;

/**
 * The transaction coordinator of two-phase commit. Runs inside the deterministic simulation ({@code testkit.SimNet}):
 * messages arrive through {@link #onMessage}, time through {@link #onTick} (once per simulated millisecond). Talks to the
 * participants only through the {@code send} function it was created with. Single-threaded.
 *
 * <p>Protocol:
 * <ol>
 *   <li>{@link #begin}: write a BEGIN record (with the participants and their writes) to the log, then send PREPARE to
 *       every participant involved. PREPAREs are re-sent every 100 ms to participants that have not voted yet.</li>
 *   <li>A NO vote decides ABORT at once. Votes of YES from all participants decide COMMIT. If no decision has been
 *       reached 500 ms after begin, ABORT is decided. The decision is appended to the log BEFORE any message about it
 *       is sent (that is the commit point), and then sent to every participant of the transaction.</li>
 *   <li>The decision is re-sent every 200 ms to participants that have not acknowledged it, until all have.</li>
 *   <li>A participant that asks about a transaction (INQUIRE) gets the decision if there is one, "ABORT" if the
 *       coordinator has never heard of the transaction (presumed abort), and no answer while the vote is still running.</li>
 *   <li>Restart: a new instance created with a non-empty log rebuilds its transactions from it. A transaction with a BEGIN
 *       but no decision is decided ABORT right away (presumed abort). Every decided transaction is announced to its
 *       participants again (acknowledgements are not persisted).</li>
 * </ol>
 * The payload format is yours.
 */
public interface CoordinatorNode {

    void onMessage(String from, String payload);

    void onTick(long nowMs);

    /**
     * Starts a transaction. {@code writesByParticipant} maps participant id to the key/value pairs it must write.
     * IllegalArgumentException for an empty map, an unknown participant or a duplicate transaction id.
     */
    void begin(String txId, Map<String, Map<String, String>> writesByParticipant);

    /** PENDING until decided. IllegalArgumentException for an unknown transaction. */
    Outcome outcome(String txId);
}

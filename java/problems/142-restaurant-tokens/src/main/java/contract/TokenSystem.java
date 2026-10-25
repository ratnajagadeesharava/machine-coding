package contract;

import java.util.List;
import java.util.Optional;
import java.util.Set;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface TokenSystem {

    /** Customer type names. */
    String DINE_IN = "DINE_IN";
    String TAKEAWAY = "TAKEAWAY";
    String VIP = "VIP";

    /** Status names returned by {@link #status(String)}. */
    String WAITING = "WAITING";
    String CALLED = "CALLED";
    String SERVED = "SERVED";
    String NO_SHOW = "NO_SHOW";
    String CANCELLED = "CANCELLED";
    String EXPIRED = "EXPIRED";

    /** Average service time (ms) used for estimates before any token has been served. */
    long DEFAULT_SERVICE_MS = 300_000L;

    /**
     * Registers a service counter that serves only the given customer types.
     * IllegalArgumentException if the id is null/blank or already registered, or the set is null, empty or holds a
     * name that is not one of DINE_IN / TAKEAWAY / VIP. Later changes to the caller's set must not affect the counter.
     */
    void addCounter(String counterId, Set<String> supportedTypes);

    /**
     * Issues the next token of the day and returns its id, {@code "T-<n>"}, with n = 1, 2, 3, ... The token is WAITING.
     * IllegalArgumentException if the name is null/blank, the type is null or not one of the three names, a DINE_IN
     * party size is outside 1..12, or a TAKEAWAY / VIP party size is not exactly 1. Nothing changes on error.
     */
    String issueToken(String customerName, String type, int partySize);

    /**
     * Serves the first token (in the current serving order) whose type the counter supports and returns it; that
     * token becomes CALLED at the current clock time. Tokens the counter does not support are skipped, not consumed.
     * Empty when no waiting token matches. IllegalArgumentException for an unknown counter.
     */
    Optional<String> callNext(String counterId);

    /** CALLED -> SERVED, recording the service duration. IllegalArgumentException unknown token, IllegalStateException otherwise. */
    void markServed(String token);

    /** CALLED -> NO_SHOW at the current clock time. IllegalArgumentException unknown token, IllegalStateException otherwise. */
    void markNoShow(String token);

    /**
     * Moves every CALLED token that has been called for MORE than graceTimeMs to NO_SHOW (stamped with the current
     * time), and every NO_SHOW token that has been no-show for MORE than recallWindowMs to EXPIRED. Returns the tokens
     * that became NO_SHOW in this call, ordered by token number.
     */
    List<String> expireOverdue();

    /**
     * NO_SHOW -> WAITING, once per token, at most recallWindowMs after it became NO_SHOW, at the front of its class.
     * IllegalArgumentException unknown token; IllegalStateException if the token is not NO_SHOW, was already recalled
     * once, or the window has passed. Nothing changes on error.
     */
    void recall(String token);

    /** WAITING -> CANCELLED. IllegalArgumentException unknown token, IllegalStateException if not WAITING. */
    void cancel(String token);

    /** One of the status names above. IllegalArgumentException for an unknown token. */
    String status(String token);

    /** 1-based position among all WAITING tokens in the serving order. IllegalArgumentException unknown token, IllegalStateException if not WAITING. */
    int positionInQueue(String token);

    /** (position - 1) * average service time, in ms. Same errors as {@link #positionInQueue(String)}. */
    long estimatedWaitMs(String token);

    /** All WAITING tokens in serving order (a copy). */
    List<String> queueSnapshot();

    /** {@code "issued:served:noShow:cancelled:waiting"}. */
    String stats();

    /** Starts a new day: forgets every token and statistic, restarts numbering at 1. Counters stay registered. */
    void resetDay();
}

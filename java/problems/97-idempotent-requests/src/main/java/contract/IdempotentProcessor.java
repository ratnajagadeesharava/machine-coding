package contract;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Time comes from the clock handed to the factory.
 */
public interface IdempotentProcessor {

    /**
     * Runs {@code handler} for the request at most once per idempotency key and returns its response; every other
     * call with the same key and the same body returns the stored response without running the handler.
     * <ul>
     *   <li>Same key, DIFFERENT body (also while the first call is still running): {@link KeyConflictException}.</li>
     *   <li>Concurrent calls with the same key and body: one runs the handler, the others wait for it and return the same
     *       response. Calls with different keys never wait for each other.</li>
     *   <li>If the handler throws, nothing is stored: the exception reaches the caller that ran it, and the key is free
     *       again. Callers that were waiting for it do not fail; one of them runs the handler next.</li>
     *   <li>A stored response expires 24 hours after it was stored (then the key can be used again).</li>
     *   <li>IllegalArgumentException for a null/blank key or a null body.</li>
     * </ul>
     */
    String process(String idempotencyKey, String requestBody, Handler handler);
}

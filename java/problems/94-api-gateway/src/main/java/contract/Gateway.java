package contract;

import java.util.List;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Time comes from the clock handed to the factory.
 */
public interface Gateway {

    void registerBackend(String id, Backend backend);

    /**
     * Routes every path that starts with {@code pathPrefix} to the backends (in the given order). When several
     * prefixes match, the longest one wins. IllegalArgumentException for an unknown backend or an empty list.
     */
    void addRoute(String pathPrefix, List<String> backendIds);

    /**
     * Registers an API key. The client may make {@code requestsPerMinute} requests per minute: a token bucket with
     * that capacity, full at registration, refilled continuously at requestsPerMinute / 60 tokens per second
     * (fractions accumulate). Each request that passes authentication and routing costs one token.
     */
    void addClient(String apiKey, String clientName, int requestsPerMinute);

    /**
     * Handles a request, checking in this order:
     * <ol>
     *   <li>unknown API key: 401 "unauthorized";</li>
     *   <li>no route: 404 "no route" (costs no token);</li>
     *   <li>no token left: 429 "rate limited";</li>
     *   <li>pick backends: the route's backends whose circuit is not open, starting with the next one in round-robin
     *       order (one shared counter per route) and failing over to the following ones when a backend throws.
     *       First success: 200 with the backend's body. If every candidate threw: 502 "bad gateway". If every
     *       backend's circuit is open: 503 "service unavailable" (nothing is called).</li>
     * </ol>
     * Circuit breaker per backend: 3 consecutive failures open it for 30 seconds; after that one trial request is let
     * through (half-open): success closes it, failure opens it for another 30 seconds. Any success resets the
     * consecutive-failure count.
     */
    Response handle(String apiKey, String path);

    /** Number of requests answered 200 for the client. */
    int successCount(String clientName);

    /** Number of times the backend was called (successful or not). */
    int callCount(String backendId);
}

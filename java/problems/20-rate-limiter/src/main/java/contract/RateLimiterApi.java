package contract;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface RateLimiterApi {

    /**
     * Decides whether a request with this key may proceed right now (time comes from the clock given at creation).
     * Allowed requests are recorded; denied requests are not.
     */
    boolean allow(String key);
}

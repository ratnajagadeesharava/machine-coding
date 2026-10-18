package contract;

import java.util.Optional;
import java.util.function.Function;

/** Thin test entry point. Your domain model lives behind it. Must be thread-safe. */
public interface CacheApi {

    /** The live value for the key, if any. */
    Optional<String> get(String key);

    void put(String key, String value);

    /**
     * Cached value, or the result of the loader (computed once for concurrent callers of the same missing key).
     * A RuntimeException thrown by the loader is propagated unchanged to every caller waiting for that computation.
     */
    String getOrCompute(String key, Function<String, String> loader);

    /** Number of live (non-expired) entries. */
    int size();
}

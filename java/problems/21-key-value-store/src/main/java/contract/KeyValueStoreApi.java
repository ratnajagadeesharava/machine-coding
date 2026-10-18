package contract;

import java.util.List;
import java.util.Optional;

/** Thin test entry point. Your domain model lives behind it. */
public interface KeyValueStoreApi {

    void put(String key, String value);

    /** Refused (false, no change) when ttlMillis <= 0. Expired once now >= setAt + ttlMillis. */
    boolean putWithTtl(String key, String value, long ttlMillis);

    Optional<String> get(String key);

    /** True only if a live key was removed. */
    boolean delete(String key);

    /** Live keys starting with the prefix, ascending. Empty prefix = all. */
    List<String> keysWithPrefix(String prefix);

    /** Number of live keys. */
    int size();

    void begin();

    /** Keeps the innermost transaction's changes; false if none is open. */
    boolean commit();

    /** Discards the innermost transaction's changes; false if none is open. */
    boolean rollback();
}

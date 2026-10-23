package contract;

import java.util.List;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface SearchCache {

    /**
     * Returns the results for the query, from the cache when possible, otherwise from the backend.
     * <p>
     * Normalisation: trim, lower-case (Locale.ROOT) and collapse every run of whitespace into one space. Word order
     * is significant ("red shoes" and "shoes red" are different queries). The normalised text is both the cache key
     * and the argument given to the backend.
     * <p>
     * A stored entry is served only if it is younger than the TTL (it expires exactly {@code ttl} ms after it was
     * stored; hits do not extend it) and the backend's {@link SearchBackend#indexVersion()} still equals the version
     * that was read just before the backend call that produced it. Otherwise the entry is dropped and the backend is
     * called. If other threads are already waiting for a backend call for the same normalised query, this call joins
     * it instead of making another one. A failed call is propagated (the same RuntimeException) to every thread that
     * waited for it and is not cached.
     * <p>
     * The returned list is unmodifiable and independent of the backend's own list.
     * IllegalArgumentException for a null or blank query (nothing is counted).
     */
    List<String> get(String rawQuery);

    /** Drops the entry for this query (normalised like {@link #get}). IllegalArgumentException for null/blank. */
    void invalidate(String rawQuery);

    /** Drops every entry. */
    void invalidateAll();

    /** Number of gets served from a stored entry. */
    long hits();

    /**
     * Number of gets that were not served from a stored entry (including the ones that joined a call already in
     * flight, and the ones whose backend call failed). A get is counted as soon as it has decided it cannot be
     * served from the cache, before it waits for or makes the backend call.
     */
    long misses();

    /** Number of times the backend's search method was invoked (failed calls included). */
    long backendCalls();

    /** Number of entries a get would serve right now (stored, not expired, index version unchanged). */
    int size();
}

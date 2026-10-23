package contract;

import java.util.Optional;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface DataAccess {

    /** Value stored under {@code key} in the current source. IllegalArgumentException for a null/blank key. */
    Optional<String> get(String key);

    /**
     * Stores or overwrites a record in the current source only. IllegalArgumentException for a null/blank key, a null
     * value, or a value the current source refuses (API: longer than 32 characters); a refused put changes nothing.
     */
    void put(String key, String value);

    /** Removes the record from the current source; true if there was one. IllegalArgumentException for a null/blank key. */
    boolean remove(String key);

    /** Number of records in the current source. */
    int size();

    /**
     * Makes {@code sourceName} (exactly "DATABASE", "API" or "FILE") the current source for every later call.
     * IllegalArgumentException for any other name (null, blank, different case); the current source is then unchanged.
     */
    void switchSource(String sourceName);

    /** Name of the current source. Before any switch it is "DATABASE". */
    String currentSource();
}

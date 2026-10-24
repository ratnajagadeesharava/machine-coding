package contract;

import java.util.List;
import java.util.Optional;

/**
 * Thin test entry point. Your design lives behind it. Every operation is atomic on its own (see the README).
 * Keys and values are never null: a null argument is an IllegalArgumentException and changes nothing.
 * The KeyHasher is called exactly once per call to put, putIfAbsent, get, getOrDefault, remove and containsKey
 * (with valid arguments), and never by any other method.
 */
public interface StringMap {

    /** Stores the value and returns the previous value for the key, or empty if the key was new. */
    Optional<String> put(String key, String value);

    /** Stores the value only if the key is absent. Returns the existing value (nothing changes) or empty (stored). */
    Optional<String> putIfAbsent(String key, String value);

    Optional<String> get(String key);

    /** The value for the key, or {@code defaultValue} if absent. Never stores anything. */
    String getOrDefault(String key, String defaultValue);

    /** Removes the key and returns its value, or empty if it was absent. */
    Optional<String> remove(String key);

    boolean containsKey(String key);

    int size();

    boolean isEmpty();

    /** Removes every entry. The capacity does not change. */
    void clear();

    /** Number of buckets. See the README for the exact growth rule. */
    int capacity();

    /** A new list of all keys in insertion order (a snapshot: later changes to the map do not affect it). */
    List<String> keys();

    /** A new list of {@code key + "=" + value} for every entry, in the same order as {@link #keys()}. */
    List<String> entries();
}

package contract;

import java.util.Optional;

/** Thin test entry point. Your design lives behind it. Must be thread-safe; keys and values are never null. */
public interface SharedCache<K, V> {

    /** Returns the value and marks the key as the most recently used. */
    Optional<V> get(K key);

    /** Returns the value WITHOUT changing the usage order (a pure read). */
    Optional<V> peek(K key);

    /** Inserts or replaces; the key becomes the most recently used. Evicts the least recently used key if over capacity. */
    void put(K key, V value);

    /** Returns true if the key was present. */
    boolean remove(K key);

    /** Number of entries; never above the capacity. */
    int size();
}

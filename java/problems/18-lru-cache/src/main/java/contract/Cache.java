package contract;

import java.util.Optional;

/** Thin test entry point. Your design lives behind it. Single-threaded. */
public interface Cache<K, V> {

    /** Present: returns the value and counts as a use. Absent: empty, nothing changes. */
    Optional<V> get(K key);

    /** Insert or replace; the key becomes the most recently used. Evicts the least recently used entry on overflow. */
    void put(K key, V value);

    /** True if an entry was removed. */
    boolean remove(K key);

    int size();
}

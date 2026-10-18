package contract;

import java.util.Optional;

/** Thin test entry point. Your design lives behind it. Single-threaded. */
public interface Cache<K, V> {

    /** Present: returns the value and counts as a use. Absent: empty, nothing changes. */
    Optional<V> get(K key);

    /** Insert (count 1) or replace (counts as a use). Evicts the least frequently, then least recently, used entry on overflow. */
    void put(K key, V value);

    /** True if an entry was removed; its use count is forgotten. */
    boolean remove(K key);

    int size();
}

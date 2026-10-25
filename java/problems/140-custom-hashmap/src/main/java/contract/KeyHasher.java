package contract;

/**
 * Provided hash function, implemented by the tests. Any int is a legal result (negative, Integer.MIN_VALUE, ...),
 * and different keys may return the same value. The map must use this and nothing else to decide where a key lives.
 */
public interface KeyHasher {

    int hash(String key);
}

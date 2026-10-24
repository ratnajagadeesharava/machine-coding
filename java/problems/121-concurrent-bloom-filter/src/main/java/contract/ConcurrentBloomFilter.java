package contract;

import java.util.Collection;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface ConcurrentBloomFilter {

    /**
     * Adds the item. Returns true if the filter changed, i.e. this call flipped at least one bit from 0 to 1;
     * false if every bit of the item was already set (a duplicate, or a new item whose bits all happen to be set).
     * Items are case-sensitive and are used exactly as given (no trimming, no lower-casing).
     * IllegalArgumentException for a null or blank (empty after trim) item; the filter is unchanged.
     */
    boolean add(String item);

    /**
     * Adds every item in iteration order and returns how many of those adds returned true. All items are validated
     * first: if the collection or any element is null/blank, IllegalArgumentException and nothing is added.
     */
    int addAll(Collection<String> items);

    /**
     * False means the item was definitely never added since the last clear. True means it was added, or it is a
     * false positive. Never changes the filter. IllegalArgumentException for a null or blank item.
     */
    boolean mightContain(String item);

    /** Estimate of the number of distinct items added: round(-(m/k) * ln(1 - X/m)), see the README. */
    long approximateSize();

    /** Number of bits currently set to 1 (X in the formula). */
    long setBits();

    /** Size of the bit array, m. */
    long bitCount();

    /** Number of bit positions per item, k. */
    int hashCount();

    /** Forgets every item: no bit is set afterwards. */
    void clear();
}

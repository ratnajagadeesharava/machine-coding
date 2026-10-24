package solution;

import contract.*;
import java.util.*;
import java.util.concurrent.*;
import java.util.function.*;

/**
 * Wiring point between the test contract and YOUR design.
 * Create your own classes anywhere under src/main/java and return them from here.
 */
public final class Solution {
    private Solution() {}

    /**
     * @param expectedItems number of distinct items the filter is sized for (must be &gt;= 1)
     * @param falsePositiveRate target false positive rate at expectedItems items (must satisfy 0 &lt; rate &lt; 1)
     * IllegalArgumentException when either is out of range.
     */
    public static ConcurrentBloomFilter create(int expectedItems, double falsePositiveRate) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

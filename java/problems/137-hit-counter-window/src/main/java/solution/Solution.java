package solution;

import contract.*;
import java.time.Clock;
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
     * @param windowMs the window length in milliseconds
     * @param clock time source (only {@code clock.millis()} is used)
     * IllegalArgumentException for {@code windowMs < 1} or a null clock.
     */
    public static HitCounter create(long windowMs, Clock clock) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

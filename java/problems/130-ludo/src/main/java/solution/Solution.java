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
     * New game for {@code players} (2..4) players that consumes {@code dice} (values 1..6) in order.
     * IllegalArgumentException for a player count outside 2..4 or a null list / a value outside 1..6.
     */
    public static LudoGame create(int players, List<Integer> dice) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

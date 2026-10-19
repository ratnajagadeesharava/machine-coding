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

    /** {@code sleeper.accept(ms)} is how you "wait" between retries (the tests record the values instead of sleeping). */
    public static SagaOrchestrator orchestrator(SagaLog log, LongConsumer sleeper) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

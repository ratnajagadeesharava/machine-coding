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

    /** Engine with your own default vector store; default chunking is 100 words with an overlap of 20. */
    public static RagEngine create(Embedder embedder) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Same engine, but vectors go to the given store. */
    public static RagEngine create(Embedder embedder, VectorStore store) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

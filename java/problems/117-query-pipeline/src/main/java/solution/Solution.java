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
     * @param dictionary  correct words (lower-case) used by the spell step
     * @param synonyms    lower-case word -> its synonyms (lower-case, single words)
     * @param bannedWords lower-case words the filter step removes
     * @param corpus      docId -> document text, searched by the rank step
     */
    public static QueryPipeline create(
            Collection<String> dictionary,
            Map<String, List<String>> synonyms,
            Collection<String> bannedWords,
            Map<String, String> corpus) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

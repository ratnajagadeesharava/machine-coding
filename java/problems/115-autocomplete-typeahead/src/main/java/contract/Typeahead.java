package contract;

import java.util.List;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Terms are stored trimmed and lower-cased (Locale.ROOT); internal whitespace is kept. Every method that takes a
 * term, prefix or user id throws IllegalArgumentException for null; a blank term or user id is also an error.
 */
public interface Typeahead {

    /** Adds 1 to the count of {@code term} and marks it as recorded now. Same as {@code record(term, 1)}. */
    void record(String term);

    /**
     * Adds {@code count} (must be positive, else IllegalArgumentException) to the count of {@code term} and marks it
     * as recorded now. The new count is visible to the very next call, from any thread. A rejected call changes nothing.
     */
    void record(String term, long count);

    /** Total count of {@code term} (0 if never recorded). Term is normalised like in {@link #record(String)}. */
    long count(String term);

    /** Switches the active ranking rule; counts and recency are kept. Null is an IllegalArgumentException. */
    void setRanking(Ranking ranking);

    /**
     * Up to {@code k} terms that start with {@code prefix} (a term equal to the prefix qualifies), best first under the
     * active ranking: FREQUENCY = count descending; RECENCY = most recently recorded first (terms recorded at the same
     * clock instant are tied). Ties are always broken alphabetically (ascending, plain character order).
     * The prefix is trimmed and lower-cased; an empty or blank prefix matches every term (top-k overall).
     * Unmatched prefix -> empty list. {@code k <= 0} -> IllegalArgumentException.
     */
    List<String> suggest(String prefix, int k);

    /**
     * Like {@link #record(String)} and additionally adds 1 to {@code userId}'s own count of the term
     * (the global count and recency are updated too).
     */
    void recordFor(String userId, String term);

    /**
     * Personalised suggestions: same candidates as {@link #suggest}, but ordered by {@code userId}'s own count of the
     * term (descending; terms the user never recorded count 0), then by the active ranking, then alphabetically.
     * A user with no history gets exactly the {@link #suggest} result.
     */
    List<String> suggestFor(String userId, String prefix, int k);
}

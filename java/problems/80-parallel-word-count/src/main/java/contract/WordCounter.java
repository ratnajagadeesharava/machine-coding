package contract;

import java.util.List;
import java.util.Map;

/** Thin test entry point. Your design lives behind it. May be called from several threads at once. */
public interface WordCounter {

    /**
     * Counts word occurrences over all documents using at most {@code parallelism} worker threads.
     *
     * <p>A word is a maximal run of ASCII letters (a-z, A-Z); every other character (digits, punctuation, spaces,
     * newlines, non-ASCII) separates words. Words are compared case-insensitively and reported in lower case.
     * The result has no zero entries; it may be a plain HashMap. The input list is not modified.
     * IllegalArgumentException if parallelism < 1.
     */
    Map<String, Integer> count(List<String> documents, int parallelism);
}

package contract;

import java.util.List;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Every word and prefix is trimmed and lower-cased (Locale.ROOT) before use. After that, a word may contain only
 * the characters a-z, 0-9 and '-'. A null argument, a blank word, or any other character is an
 * IllegalArgumentException. A blank prefix is valid and means "everything".
 */
public interface WordSet {

    /** Adds the word. Returns true if it was not present before, false if it already was. */
    boolean insert(String word);

    /** True if exactly this word is present. */
    boolean contains(String word);

    /** True if at least one present word starts with the prefix (a word equal to the prefix counts). */
    boolean startsWith(String prefix);

    /** Number of present words that start with the prefix (a word equal to the prefix counts). */
    int countWithPrefix(String prefix);

    /**
     * Up to {@code limit} present words that start with the prefix, in ascending lexicographic (plain character) order,
     * without duplicates. {@code limit <= 0} is an IllegalArgumentException. The list is a fresh copy.
     */
    List<String> wordsWithPrefix(String prefix, int limit);

    /** Removes the word. Returns true if it was present. Other words that share its prefix are not affected. */
    boolean delete(String word);

    /** Number of words currently present. */
    int size();
}

package contract;

import java.util.List;
import java.util.Optional;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Every method that takes a word trims it, lower-cases it and requires it to be non-blank and made only of the
 * ASCII letters a-z (any case), '-' and the apostrophe '. Anything else (including null) is an
 * IllegalArgumentException. All other errors are IllegalArgumentException too. Sorted always means ascending by
 * plain character code (so the apostrophe sorts before '-', which sorts before the letters).
 */
public interface DictionaryApp {

    /**
     * Adds a meaning to the word (creating the word if needed). {@code partOfSpeech} is NOUN, VERB, ADJECTIVE or
     * ADVERB (any case, surrounding whitespace ignored); {@code meaning} is trimmed and must not be blank.
     * Returns false and changes nothing when the word already has exactly this (part of speech, meaning) pair
     * (meaning compared case-sensitively); true otherwise. Invalid input is an error and changes nothing.
     */
    boolean addWord(String word, String partOfSpeech, String meaning);

    /** The word's meanings in insertion order, each as {@code "POS: meaning"} (POS in upper case). Unknown word: empty list. */
    List<String> lookup(String word);

    /**
     * Replaces the meaning text at the 0-based {@code index}, keeping its part of speech and position.
     * Errors: unknown word, index outside the word's meanings, blank newMeaning, or newMeaning (trimmed) equal to the
     * meaning of a different entry with the same part of speech. Re-applying the entry's own text is allowed.
     */
    void updateMeaning(String word, int index, String newMeaning);

    /**
     * Removes the meaning at the 0-based index; later meanings shift down by one. Removing a word's last meaning
     * removes the word itself. Errors: unknown word, index outside the word's meanings.
     */
    void removeMeaning(String word, int index);

    /** Removes the word with all its meanings and synonym links. Returns whether the word existed. */
    boolean removeWord(String word);

    /**
     * Links two words as direct synonyms (symmetric; links are not transitive). Returns false if they were already
     * linked, true otherwise. Errors: invalid word, word equal to other (after normalising), either word unknown.
     */
    boolean addSynonym(String word, String other);

    /** The direct synonyms of the word, sorted. Unknown word: empty list. */
    List<String> synonyms(String word);

    /** Words starting with the (normalised, non-blank) prefix, sorted, at most {@code limit}. limit <= 0 is an error. */
    List<String> prefix(String prefix, int limit);

    /**
     * Words having at least one meaning that contains {@code text} (trimmed, non-blank, compared case-insensitively),
     * sorted by word, at most {@code limit}. Only meaning texts are searched. limit <= 0 is an error.
     */
    List<String> search(String text, int limit);

    /**
     * If the word is not in the dictionary: up to {@code k} existing words at Levenshtein distance exactly 1
     * (one insertion, deletion or substitution), sorted. A known word gives an empty list. k <= 0 is an error.
     */
    List<String> suggest(String word, int k);

    /** The word at index floorMod(dayNumber, size()) in sorted order, or empty when the dictionary is empty. */
    Optional<String> wordOfTheDay(long dayNumber);

    /** Number of words. */
    int size();

    /** Total number of meaning entries over all words. */
    int meaningCount();
}

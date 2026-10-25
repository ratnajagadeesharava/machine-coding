package contract;

import java.util.List;
import java.util.Map;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>All positions are character indexes (the tests use ASCII text only). Every error is an
 * {@link IllegalArgumentException} and leaves the document unchanged.
 */
public interface WordProcessor {

    /**
     * Inserts {@code text} before character {@code pos} (0 &lt;= pos &lt;= length(); {@code pos == length()} appends).
     * The new characters take the format of the character before {@code pos}; at position 0, or in an empty document,
     * they take the default format ("Calibri", 11, not bold, not italic, not underlined, "black").
     * Inserting an empty string is allowed and changes nothing. Null text is an error.
     */
    void insert(int pos, String text);

    /** Deletes the half-open range [from, to). Requires 0 &lt;= from &lt;= to &lt;= length(); an empty range changes nothing. */
    void delete(int from, int to);

    /**
     * Changes the format of the characters in [from, to) (same range rules as {@link #delete}). {@code change} holds
     * only the fields to change; the other fields keep their values. Recognised keys and values:
     * "font" (non-blank, no '|'), "size" (integer 1..1000), "bold" / "italic" / "underline" ("true" or "false"),
     * "color" (non-blank, no '|'). An unknown key or an invalid value is an error. An empty map changes nothing.
     * Nothing is applied if anything is invalid.
     */
    void applyFormat(int from, int to, Map<String, String> change);

    /** The format of the character at {@code pos} (0 &lt;= pos &lt; length()) as "font|size|B|I|U|color", e.g. "Calibri|11|-|-|-|black" or "Arial|14|B|-|U|red". */
    String formatAt(int pos);

    /** The whole text. */
    String text();

    /** Number of characters. */
    int length();

    /** Maximal runs of equal format, in document order, as "start-end:formatString" (end exclusive), e.g. "0-5:Calibri|11|-|-|-|black". Empty document: empty list. */
    List<String> runs();

    /** Start indexes (ascending) of every match, overlapping ones included. An empty or null pattern is an error. */
    List<Integer> find(String pattern, boolean caseSensitive);

    /**
     * Replaces the non-overlapping matches found scanning left to right in the original text and returns how many
     * were replaced. Every character of a replacement takes the format of the first character of the match it replaced.
     * An empty or null pattern, or a null replacement, is an error.
     */
    int replaceAll(String pattern, String replacement, boolean caseSensitive);

    /** Number of maximal sequences of non-whitespace characters (whitespace: space, tab, '\n', '\r'). */
    int wordCount();

    /** Number of '\n' characters plus one. */
    int lineCount();

    /** Sets the alignment of a line (0 &lt;= line &lt; lineCount()); alignment is one of "LEFT", "CENTER", "RIGHT", "JUSTIFY". */
    void setAlignment(int line, String alignment);

    /** The alignment of a line (0 &lt;= line &lt; lineCount()); "LEFT" unless set or inherited. */
    String alignment(int line);

    /** Number of distinct formats currently used by at least one character (0 for an empty document). */
    int distinctFormatCount();
}

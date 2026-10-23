package contract;

import java.util.List;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface SearchEngine {

    /**
     * Adds a document. IllegalArgumentException if {@code docId} is null/blank, {@code text} is null, or a live
     * document with that id already exists. Empty text (or text without any token) is allowed.
     */
    void addDocument(String docId, String text);

    /** Replaces the text of a live document; visible to the very next search. IllegalArgumentException if unknown or text is null. */
    void updateDocument(String docId, String text);

    /** Removes a live document; visible to the very next search. IllegalArgumentException if unknown. The id may be added again later. */
    void deleteDocument(String docId);

    /**
     * Same as {@code search(query, mode, 0, k)}: the {@code k} best documents. {@code mode} is exactly "AND" or "OR".
     * IllegalArgumentException for a null query, any other mode, or k &lt; 0.
     */
    List<String> search(String query, String mode, int k);

    /**
     * The documents ranked {@code offset} .. {@code offset + limit - 1} of the full ranking (best first), as doc ids.
     * Ranking: TF-IDF score descending, ties by doc id ascending (formula in the README). Fewer (or none) are
     * returned when the ranking is shorter. IllegalArgumentException for a null query, a mode other than "AND"/"OR",
     * offset &lt; 0 or limit &lt; 0.
     */
    List<String> search(String query, String mode, int offset, int limit);

    /**
     * TF-IDF score of a live document for the query: the sum over the distinct query terms that occur in the document,
     * independent of the mode. 0.0 when no query term occurs in it. IllegalArgumentException for a null query or an
     * unknown document.
     */
    double scoreOf(String query, String docId);

    /** Number of live documents. */
    int documentCount();
}

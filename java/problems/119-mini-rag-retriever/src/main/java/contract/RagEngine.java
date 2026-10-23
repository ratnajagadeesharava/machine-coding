package contract;

import java.util.List;
import java.util.Optional;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface RagEngine {

    /**
     * Splits {@code text} on whitespace into words and cuts it into chunks of at most chunkSize words, consecutive
     * chunks sharing {@code overlap} words (see README). Stores the chunks under ids {@code docId + "#" + n}
     * (n from 0) and returns how many were created. Empty or blank text creates 0 chunks. Re-ingesting a docId
     * replaces all its previous chunks as one atomic step. IllegalArgumentException for a null/blank docId or null text.
     * If the embedder throws, the exception propagates and nothing changes.
     */
    int ingest(String docId, String text);

    /**
     * Chunk size and overlap (in words) used by LATER ingests; chunks already stored are untouched.
     * IllegalArgumentException unless chunkSize &gt;= 1 and 0 &lt;= overlap &lt; chunkSize (state unchanged then).
     */
    void setChunking(int chunkSize, int overlap);

    /** Removes every chunk of the document; returns false if the document was unknown. Blank/null docId is an error. */
    boolean deleteDocument(String docId);

    /**
     * Ids of the {@code min(k, chunkCount())} chunks most similar (cosine) to the query, best first, ties by chunk id
     * ascending (String.compareTo). IllegalArgumentException for k &lt; 1 or a null/blank query.
     */
    List<String> retrieve(String query, int k);

    /**
     * Prompt for the language model; the exact template is in the README. Uses the chunks {@code retrieve(query, k)}
     * would return, whole chunks only, within a context budget of {@code maxContextChars}.
     * IllegalArgumentException for k &lt; 1, a null/blank query or a negative budget.
     */
    String buildPrompt(String query, int k, int maxContextChars);

    /** Number of chunks currently stored. */
    int chunkCount();

    /** The text of a stored chunk, or empty if there is no such chunk. */
    Optional<String> chunkText(String chunkId);
}

package contract;

import java.util.List;

/**
 * Stores vectors and finds the ones most similar to a query. The tests supply an alternative implementation
 * (a plain brute-force scan) to prove the store can be swapped; your component must also work without one.
 * Implementations are thread-safe.
 */
public interface VectorStore {

    /** One search result: the stored id, its cosine similarity to the query, and the payload stored with it. */
    record Hit(String id, double score, String payload) {}

    /** Inserts the vector, or replaces the vector and payload already stored under {@code id}. */
    void upsert(String id, double[] vector, String payload);

    /** Removes the entry; returns true if it existed. */
    boolean remove(String id);

    /**
     * At most {@code k} hits (k &gt;= 1), most similar first by cosine similarity; equal scores are ordered by id
     * ascending (String.compareTo). A zero vector (query or stored) has similarity 0 with everything.
     */
    List<Hit> topK(double[] query, int k);
}

package solution;

import contract.DocumentStore;
import java.util.Optional;

/**
 * Caching proxy: remembers documents the real store has returned so the (slow) real store is asked only once per
 * document. A blank/null id -> IllegalArgumentException. Only PRESENT documents are cached: a missing document is
 * looked up again next time (it may have been created meanwhile).
 */
public class CachingStoreProxy implements DocumentStore {

    public CachingStoreProxy(DocumentStore real) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public Optional<String> fetch(String id) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Forgets the cached copy of one document (no effect if it is not cached). */
    public void invalidate(String id) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Number of documents currently cached. */
    public int cachedCount() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

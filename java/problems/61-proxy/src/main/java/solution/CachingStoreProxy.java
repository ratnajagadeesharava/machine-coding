package solution;

import contract.DocumentStore;
import java.util.Optional;

/** Caching proxy. A blank/null id -> IllegalArgumentException. */
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

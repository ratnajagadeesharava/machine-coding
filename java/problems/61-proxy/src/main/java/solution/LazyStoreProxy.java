package solution;

import contract.DocumentStore;
import java.util.Optional;
import java.util.function.Supplier;

/** Virtual proxy: creates the real store only when the first fetch needs it. */
public class LazyStoreProxy implements DocumentStore {

    public LazyStoreProxy(Supplier<DocumentStore> factory) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public Optional<String> fetch(String id) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

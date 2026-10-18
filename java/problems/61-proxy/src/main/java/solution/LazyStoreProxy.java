package solution;

import contract.DocumentStore;
import java.util.Optional;
import java.util.function.Supplier;

/**
 * Virtual proxy: the real store is expensive to build (it opens connections), so it is created only when the first
 * fetch needs it, and then reused. Constructing the proxy must NOT call the factory.
 */
public class LazyStoreProxy implements DocumentStore {

    public LazyStoreProxy(Supplier<DocumentStore> factory) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public Optional<String> fetch(String id) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

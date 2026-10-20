package solution;

import contract.AccessPolicy;
import contract.DocumentStore;
import java.util.Optional;

/** Protection proxy: denied access -> SecurityException. A blank/null id -> IllegalArgumentException. */
public class AccessControlStoreProxy implements DocumentStore {

    public AccessControlStoreProxy(DocumentStore real, AccessPolicy policy, String user) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public Optional<String> fetch(String id) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

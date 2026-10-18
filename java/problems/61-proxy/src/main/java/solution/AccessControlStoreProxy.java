package solution;

import contract.AccessPolicy;
import contract.DocumentStore;
import java.util.Optional;

/**
 * Protection proxy: checks {@code policy.canRead(user, id)} before every fetch. If not allowed it throws
 * SecurityException and the real store is NEVER asked. A blank/null id -> IllegalArgumentException (policy not asked).
 */
public class AccessControlStoreProxy implements DocumentStore {

    public AccessControlStoreProxy(DocumentStore real, AccessPolicy policy, String user) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public Optional<String> fetch(String id) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

package contract;

import java.util.List;

/**
 * The search service behind the cache. PROVIDED: the tests implement it (a counting fake). Not part of your design.
 * Must be safe to call from several threads at once.
 */
public interface SearchBackend {

    /**
     * Runs the query and returns the matching document ids, best first. The argument is already normalised. The
     * call may be slow and may throw a RuntimeException. Never returns null. The returned list may be mutable and
     * the backend may keep changing it afterwards.
     */
    List<String> search(String normalisedQuery);

    /**
     * The current version of the index. It changes (it only ever goes up) whenever documents are added, changed or
     * removed. Cheap to call.
     */
    long indexVersion();
}

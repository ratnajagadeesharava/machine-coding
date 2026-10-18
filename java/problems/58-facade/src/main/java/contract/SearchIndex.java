package contract;

/** Provided subsystem. {@code index} may throw a RuntimeException when the search cluster is down. */
public interface SearchIndex {

    void index(String url, String fileName);
}

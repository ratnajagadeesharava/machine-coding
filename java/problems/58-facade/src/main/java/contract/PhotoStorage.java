package contract;

/** Provided subsystem. {@code store} may throw a RuntimeException when the disk or network fails. */
public interface PhotoStorage {

    /** Stores the file and returns its URL. */
    String store(String fileName, String content);

    void delete(String url);
}

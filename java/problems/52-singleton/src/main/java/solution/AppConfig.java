package solution;

import java.util.Optional;

/**
 * The application-wide configuration. Exactly one instance exists in the JVM, it is created lazily (on the first
 * call to {@link #getInstance()}), creation is thread-safe, and nobody outside this class can construct one.
 */
public final class AppConfig {

    private AppConfig() {
        // The tests count how many times your constructor runs: make it increase the counter returned by
        // instancesCreated().
        this.config = new HashMap<>();

    }

    /**
     * The single instance. Many threads may call this at the same moment; still only ONE instance is ever created.
     */
    public static AppConfig getInstance() {
        if (instance == null) {
            noOfInstances = 0;
            instance = new AppConfig();
        noOfInstances++;
        }
        return instance;

    }

    /** How many instances have been constructed since the last {@link #resetForTests()} (0 before the first getInstance). */
    public static int instancesCreated() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Test hook: forget the current instance and reset the counter to 0. The next getInstance() builds a fresh one. */
    public static void resetForTests() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Null key -> IllegalArgumentException. Safe to call from many threads. */
    public void set(String key, String value) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * Empty when the key was never set. Null key -> IllegalArgumentException.
     */
    public Optional<String> get(String key) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

package solution;

import java.util.Optional;

/** The application-wide configuration. */
public final class AppConfig {

    private AppConfig() {}

    public static AppConfig getInstance() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public static int instancesCreated() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Test hook. */
    public static void resetForTests() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public void set(String key, String value) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public Optional<String> get(String key) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

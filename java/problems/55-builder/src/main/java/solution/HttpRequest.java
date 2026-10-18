package solution;

import java.util.Map;
import java.util.Optional;

/**
 * An immutable HTTP request description built step by step. There is no public constructor and no setter:
 * the only way to get one is {@code HttpRequest.builder()...build()}.
 */
public final class HttpRequest {

    public static Builder builder() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** "GET", "POST", ... (upper case). */
    public String method() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** The URL including the query string, e.g. {@code https://api.test/items?page=2&q=a%20b}. */
    public String url() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Header names are lower-case. Insertion order. The returned map cannot be modified. */
    public Map<String, String> headers() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Case-insensitive lookup. */
    public Optional<String> header(String name) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public Optional<String> body() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public int timeoutMillis() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * Fluent builder. Defaults: method GET, timeout 5000 ms. Every setter returns the builder.
     * Setters reject bad arguments immediately with IllegalArgumentException; build() reports an incomplete or
     * inconsistent request with IllegalStateException. A builder can be reused: each build() creates an independent
     * request, and changing the builder afterwards never changes requests it already built.
     */
    public static final class Builder {

        /** GET, POST, PUT, PATCH or DELETE in any letter case; anything else is rejected. */
        public Builder method(String method) {
            throw new UnsupportedOperationException("Not implemented yet");
        }

        /** Must start with "http://" or "https://" followed by something. */
        public Builder url(String url) {
            throw new UnsupportedOperationException("Not implemented yet");
        }

        /** Blank name or null value rejected. Setting the same name again (any case) replaces the value in place. */
        public Builder header(String name, String value) {
            throw new UnsupportedOperationException("Not implemented yet");
        }

        /**
         * Adds {@code name=value} to the query string, in call order. Name and value are percent-encoded: every UTF-8
         * byte other than A-Z a-z 0-9 '-' '_' '.' '~' becomes %XX (upper-case hex), so a space is %20. The query is
         * appended with '?' (or '&amp;' if the URL already contains '?'). Blank name rejected.
         */
        public Builder query(String name, String value) {
            throw new UnsupportedOperationException("Not implemented yet");
        }

        public Builder body(String body) {
            throw new UnsupportedOperationException("Not implemented yet");
        }

        /** 1 to 60000 inclusive. */
        public Builder timeoutMillis(int millis) {
            throw new UnsupportedOperationException("Not implemented yet");
        }

        /** No URL set -> IllegalStateException. A body together with GET or DELETE -> IllegalStateException. */
        public HttpRequest build() {
            throw new UnsupportedOperationException("Not implemented yet");
        }
    }
}

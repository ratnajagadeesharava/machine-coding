package solution;

import java.util.Map;
import java.util.Optional;

/** An HTTP request description. */
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

    /** Request headers. */
    public Map<String, String> headers() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Header value by name. */
    public Optional<String> header(String name) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public Optional<String> body() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public int timeoutMillis() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Builder for {@link HttpRequest}. */
    public static final class Builder {

        public Builder method(String method) {
            throw new UnsupportedOperationException("Not implemented yet");
        }

        public Builder url(String url) {
            throw new UnsupportedOperationException("Not implemented yet");
        }

        public Builder header(String name, String value) {
            throw new UnsupportedOperationException("Not implemented yet");
        }

        public Builder query(String name, String value) {
            throw new UnsupportedOperationException("Not implemented yet");
        }

        public Builder body(String body) {
            throw new UnsupportedOperationException("Not implemented yet");
        }

        public Builder timeoutMillis(int millis) {
            throw new UnsupportedOperationException("Not implemented yet");
        }

        public HttpRequest build() {
            throw new UnsupportedOperationException("Not implemented yet");
        }
    }
}

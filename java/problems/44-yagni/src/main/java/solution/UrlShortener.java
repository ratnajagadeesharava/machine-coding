package solution;

import java.util.Optional;

/** Does exactly what the README asks and nothing more. */
public class UrlShortener {

    public UrlShortener() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * Returns the short code for a URL. Codes are the sequence numbers 1, 2, 3, ... written in lower-case base 36
     * ("1" ... "9", "a" ... "z", "10", ...). The same URL always gets the same code. The URL is trimmed first.
     * Anything that does not start with "http://" or "https://" followed by at least one character
     * -> IllegalArgumentException.
     */
    public String shorten(String url) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** The original (trimmed) URL for a code, or empty if the code is unknown. */
    public Optional<String> resolve(String code) {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

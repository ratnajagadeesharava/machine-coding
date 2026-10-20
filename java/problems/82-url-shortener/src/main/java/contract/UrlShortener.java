package contract;

import java.util.Optional;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface UrlShortener {

    /**
     * Returns the short code for an http(s) URL. Shortening the same URL again returns the same code.
     * IllegalArgumentException if the URL is null, blank or not an http:// or https:// URL.
     */
    String shorten(String longUrl);

    /**
     * Like {@link #shorten(String)} but with a caller-chosen code. The alias must match [A-Za-z0-9_-]{3,20}.
     * Re-registering the same alias for the same URL returns it again. IllegalArgumentException for a bad URL,
     * a bad alias, or an alias that already means a different URL (or is an existing generated code).
     */
    String shorten(String longUrl, String alias);

    /** The URL behind a code, counting one click. Empty (and no click) for an unknown or deleted code. */
    Optional<String> resolve(String code);

    /** Number of successful resolves. IllegalArgumentException for an unknown or deleted code. */
    int clicks(String code);

    /** Removes a code. Returns false if it did not exist. A deleted code is never handed out again. */
    boolean delete(String code);
}

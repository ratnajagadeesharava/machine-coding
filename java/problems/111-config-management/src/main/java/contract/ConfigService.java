package contract;

import java.util.List;
import java.util.Optional;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>A config record is reported as a single string: {@code <namespace>/<key>=<value> v<version> t<updatedAt>}
 * where {@code updatedAt} is the injected clock's epoch milliseconds at the moment that version was written.
 * Example: {@code app/db.host=localhost v2 t1500}. Call such a string a "view" below.
 *
 * <p>Errors: {@link IllegalArgumentException} for invalid input, {@link IllegalStateException} for creating a
 * config that already exists, {@link java.util.NoSuchElementException} for changing a config that does not exist.
 * Argument validation always happens first, before existence and version checks, and a failed call changes nothing.
 */
public interface ConfigService {

    /**
     * Creates a config at version 1 with {@code updatedAt} = now.
     * IllegalArgumentException for a blank/null namespace, a key that is empty or has a character outside
     * {@code [a-z0-9._-]}, or a null value or one longer than 256 characters (an empty value is allowed).
     * IllegalStateException if (namespace, key) already exists.
     */
    void create(String namespace, String key, String value);

    /**
     * Replaces the value if and only if the current version equals {@code expectedVersion}; the version then goes up
     * by exactly 1 (even when the new value equals the old one) and {@code updatedAt} = now. Returns true on success.
     * Returns false, changing nothing, when the current version differs from {@code expectedVersion}.
     * IllegalArgumentException for invalid input (same rules as create); NoSuchElementException if the config does
     * not exist.
     */
    boolean update(String namespace, String key, String value, long expectedVersion);

    /** The view of the current version, or empty if the config does not exist. IllegalArgumentException for an invalid namespace or key. */
    Optional<String> get(String namespace, String key);

    /**
     * Removes the config together with its history; returns true if it existed. Creating it again starts over at
     * version 1. IllegalArgumentException for an invalid namespace or key.
     */
    boolean delete(String namespace, String key);

    /**
     * Views of the current version of every config in the namespace, ascending by key (plain string order).
     * Empty for an unknown namespace. IllegalArgumentException for a blank/null namespace.
     */
    List<String> list(String namespace);

    /**
     * Views of every version the config has had, oldest first, including the current one (the last element equals
     * what {@code get} returns). Empty if the config does not exist. The returned list is a snapshot: later changes
     * do not alter it. IllegalArgumentException for an invalid namespace or key.
     */
    List<String> history(String namespace, String key);

    /**
     * Creates a NEW version (current version + 1, {@code updatedAt} = now) whose value is the value that
     * {@code toVersion} had. Earlier history is untouched. Rolling back to the current version is allowed and still
     * creates a new version. Returns the new version number.
     * IllegalArgumentException for an invalid namespace or key, or a {@code toVersion} that never existed for this
     * config (below 1 or above the current version); NoSuchElementException if the config does not exist.
     */
    long rollback(String namespace, String key, long toVersion);
}

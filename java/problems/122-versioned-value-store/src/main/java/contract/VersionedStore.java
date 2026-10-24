package contract;

import java.util.List;
import java.util.Map;
import java.util.Optional;
import java.util.OptionalLong;

/** Thin test entry point. Your design lives behind it. Must be thread-safe. */
public interface VersionedStore {

    /**
     * Writes {@code value} for {@code key} and returns the new store-wide version (1, 2, 3, ... without gaps).
     * IllegalArgumentException if the key is null/blank or the value is null (no version is consumed).
     */
    long put(String key, String value);

    /**
     * Writes all entries as ONE version and returns it: a reader sees all of them or none of them.
     * IllegalArgumentException if the map is null or empty, or any key is null/blank, or any value is null;
     * nothing is applied and no version is consumed.
     */
    long putAll(Map<String, String> entries);

    /**
     * Deletes the key by writing a tombstone and returns its version, or -1 if the key has no live value
     * (never written, or already deleted); in that case nothing changes and no version is consumed.
     */
    long delete(String key);

    /** The latest value, empty if the key is deleted or was never written. */
    Optional<String> get(String key);

    /**
     * The value of {@code key} as of {@code version}: the newest write with version &lt;= {@code version}
     * (empty if there is none or it is a tombstone). IllegalArgumentException if {@code version} is below 1 or above
     * {@link #currentVersion()}; IllegalStateException if it is below the compaction floor (see {@link #compact}).
     */
    Optional<String> getAt(String key, long version);

    /** Version of the latest successful write; 0 for a store that has never been written. */
    long currentVersion();

    /**
     * Retained history of the key, oldest first, one string per entry: {@code "<version>:<value>"}, and
     * {@code "<version>:<deleted>"} for a tombstone. Empty for an unknown key.
     */
    List<String> history(String key);

    /**
     * Compare-and-set. Succeeds only if {@code expectedKeyVersion} equals the version of the key's current live
     * value, where 0 means "the key has no live value" (never written, or deleted). On success returns the new
     * store-wide version; on mismatch returns empty, changes nothing and consumes no version.
     * IllegalArgumentException for invalid input or a negative expected version.
     */
    OptionalLong putIfVersion(String key, String value, long expectedKeyVersion);

    /** Opens a read view fixed at {@link #currentVersion()} right now and returns its id (a positive number, unique). */
    long snapshot();

    /** The value of the key in that snapshot's view. IllegalArgumentException for an unknown or released id. */
    Optional<String> snapshotGet(long snapshotId, String key);

    /** Closes the snapshot. IllegalArgumentException for an unknown or already released id. */
    void releaseSnapshot(long snapshotId);

    /**
     * Drops history that no read can need any more and returns how many entries were removed. Reads that stay
     * answerable: getAt for every version &gt;= {@code keepVersionsAtOrAfter}, get, and every open snapshot.
     * Afterwards getAt for a version below the floor (the highest value passed so far) throws IllegalStateException.
     * IllegalArgumentException unless 1 &lt;= keepVersionsAtOrAfter &lt;= currentVersion().
     */
    long compact(long keepVersionsAtOrAfter);
}

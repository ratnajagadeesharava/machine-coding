package contract;

import java.util.List;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>A document is text; its lines are {@code content.split("\n", -1)} and are joined again with "\n".
 * Versions are numbered 1, 2, 3 ... and are never rewritten (history is append-only).
 */
public interface DocStore {

    /** Creates version 1. IllegalArgumentException if the document already exists. */
    void create(String docId, String author, String content);

    /** Number of the newest version. IllegalArgumentException for an unknown document. */
    int latest(String docId);

    /** Content of a version. IllegalArgumentException for an unknown document or version. */
    String content(String docId, int version);

    /**
     * Saves {@code newContent}, which the author edited starting from {@code baseVersion}, and returns the version
     * that now holds the result.
     * <ul>
     *   <li>base == latest: if the content is unchanged nothing is created and latest is returned; else a new version.</li>
     *   <li>base &lt; latest (stale): three-way merge of base, latest and newContent (see the README for the exact
     *       rules). A clean merge creates a new version with the merged content (or returns latest if the merge equals
     *       latest); an overlapping change throws {@link MergeConflictException} and creates nothing.</li>
     *   <li>IllegalArgumentException if base is not in [1, latest] or the document is unknown.</li>
     * </ul>
     */
    int edit(String docId, String author, int baseVersion, String newContent);

    /** Creates a new version holding the content of an old one (returns latest, creating nothing, if equal). */
    int restore(String docId, String author, int version);

    /** One entry per version, oldest first, formatted "version:author". */
    List<String> history(String docId);

    /**
     * Line diff from one version to another, one entry per output line: "  text" (unchanged), "- text" (removed),
     * "+ text" (added). Exact algorithm in the README.
     */
    List<String> diff(String docId, int fromVersion, int toVersion);
}

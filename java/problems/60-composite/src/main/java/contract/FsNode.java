package contract;

import java.util.List;

/** Provided component interface: the same operations work on a single file or on a whole folder tree. */
public interface FsNode {

    /** Name of this node (no '/'). */
    String name();

    /** Total size in bytes (for a folder: everything inside it, recursively). */
    long size();

    /** Number of files (for a folder: all files inside it, recursively; a plain file counts 1). */
    int fileCount();

    /**
     * Paths of all files, relative to the PARENT of this node: a file gives {@code [name]}; a folder gives, for each
     * child path {@code p}, {@code name + "/" + p}, in child insertion order (an empty folder gives an empty list).
     */
    List<String> paths();
}

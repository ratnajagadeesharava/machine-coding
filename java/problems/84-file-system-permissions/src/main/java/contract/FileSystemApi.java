package contract;

import java.util.List;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Failures: IllegalArgumentException for a bad path, a missing/existing entry, a wrong entry type, a non-empty
 * directory or an unknown user; SecurityException when the user lacks the required permission.
 */
public interface FileSystemApi {

    /** Registers a user. The first group is the primary group. The user "root" always exists and may do anything. */
    void addUser(String user, List<String> groups);

    /** New directory: owner = user, group = user's primary group, mode 0755. Needs w+x on the parent. */
    void mkdir(String user, String path);

    /** New file: owner = user, group = primary group, mode 0644. Needs w+x on the parent. */
    void createFile(String user, String path, String content);

    /** Needs r on the file. */
    String read(String user, String path);

    /** Replaces the content of an existing file. Needs w on the file. */
    void write(String user, String path, String content);

    /** Names of the entries, sorted. Needs r on the directory. */
    List<String> list(String user, String path);

    /** Removes a file or an EMPTY directory. Needs w+x on the parent. */
    void delete(String user, String path);

    /** mode is the classic octal permission value (0755 is rwxr-xr-x). Only the owner or root. */
    void chmod(String user, String path, int mode);

    /** Only root. */
    void chown(String user, String path, String newOwner, String newGroup);

    /** Renames/moves a file or directory; the target must not exist. Needs w+x on both parents. */
    void move(String user, String from, String to);

    /** Unchecked inspection helpers (no permission needed). IllegalArgumentException if the path does not exist. */
    int mode(String path);

    String owner(String path);

    String group(String path);
}

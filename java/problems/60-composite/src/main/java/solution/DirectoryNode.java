package solution;

import contract.FsNode;
import java.util.List;
import java.util.Optional;

/** The composite: a folder that contains {@link FsNode}s of any kind, including other folders. */
public class DirectoryNode implements FsNode {

    /** Blank name or a name containing '/' -> IllegalArgumentException. */
    public DirectoryNode(String name) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * Adds a child. Null, a child whose name is already used in this folder, or a folder that would create a cycle
     * (this folder itself, or any folder that contains this one) -> IllegalArgumentException.
     */
    public void add(FsNode child) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Removes the child with that name. Returns whether it existed. */
    public boolean remove(String name) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Children in insertion order. Callers cannot modify the folder through the returned list. */
    public List<FsNode> children() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /**
     * Looks a node up by a '/'-separated path RELATIVE to this folder, e.g. {@code "docs/b.txt"}. Empty when any
     * step is missing or a step goes through a file. Blank path -> IllegalArgumentException.
     */
    public Optional<FsNode> find(String path) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public String name() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public long size() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public int fileCount() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public List<String> paths() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

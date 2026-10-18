package solution;

import contract.FsNode;
import java.util.List;

/** The leaf. Blank name or a name containing '/' -> IllegalArgumentException; negative size -> IllegalArgumentException. */
public class FileNode implements FsNode {

    public FileNode(String name, long sizeBytes) {
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

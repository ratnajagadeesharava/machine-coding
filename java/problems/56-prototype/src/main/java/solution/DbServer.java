package solution;

import contract.Blueprint;
import java.util.List;

/** A database server blueprint. */
public class DbServer implements Blueprint {

    public DbServer(String name) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public String name() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public void rename(String newName) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public String kind() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public void addTag(String tag) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public List<String> tags() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public List<Integer> diskSizesGb() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public void resizeDisk(int index, int newSizeGb) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    @Override
    public Blueprint copy() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

package solution;

import contract.Blueprint;
import java.util.List;

/** A web server blueprint. kind() is "web"; it starts with one 20 GB disk and no tags. Blank name -> IllegalArgumentException. */
public class WebServer implements Blueprint {

    public WebServer(String name) {
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

package solution;

import java.util.List;

/** A playlist groups songs but does not own them. */
public class Playlist {

    public Playlist(String name) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public String name() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Adds the very same Song instance (no copy). Returns false if that instance is already in; null -> IllegalArgumentException. */
    public boolean add(Song song) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Removes that instance. Returns whether it was present. The song itself is untouched. */
    public boolean remove(Song song) {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Songs in the order they were added; the same instances that were added. Callers cannot modify the list. */
    public List<Song> songs() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    public int totalSeconds() {
        throw new UnsupportedOperationException("Not implemented yet");
    }

    /** Empties the playlist. The songs live on. */
    public void clear() {
        throw new UnsupportedOperationException("Not implemented yet");
    }
}

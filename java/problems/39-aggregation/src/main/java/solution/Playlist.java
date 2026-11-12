package solution;

import java.util.ArrayList;
import java.util.List;

/**
 * A playlist groups songs but does not own them.
 */
public class Playlist {
    private final String name;
    private List<Song> songs;

    public Playlist(String name) {
        if (name == null || name.isBlank()) {
            throw new IllegalArgumentException("name cannot be empty");
        }
        this.name = name;
        this.songs = new ArrayList<>();
    }

    public String name() {
        return this.name;
    }

    /**
     * Adds the very same Song instance (no copy). Returns false if that instance is already in; null -> IllegalArgumentException.
     */
    public boolean add(Song song) {
        if (song == null) {
            throw new IllegalArgumentException("song cannot be null");
        }
         if(!songs.contains(song)){
             this.songs.add(song);
             return true;
         }
         else {
             return false;
         }
    }

    /**
     * Removes that instance. Returns whether it was present. The song itself is untouched.
     */
    public boolean remove(Song song) {
        if (song == null) {
            throw new IllegalArgumentException("song cannot be null");
        }
        if (!songs.contains(song)) return false;
        songs.remove(song);
        return true;
    }

    /**
     * Songs in the order they were added; the same instances that were added. Callers cannot modify the list.
     */
    public List<Song> songs() {
        return List.copyOf(songs);
    }

    public int totalSeconds() {
        int count = songs.stream().map((song) -> song.seconds()).reduce(0, (cnt, song) -> cnt = cnt + song);
        return count;
    }

    /**
     * Empties the playlist. The songs live on.
     */
    public void clear() {
        this.songs.clear();

    }
}

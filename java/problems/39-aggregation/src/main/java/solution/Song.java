package solution;

import java.util.Objects;

/**
 * A song exists on its own, independent of any playlist.
 */
public class Song {
    private String title;
    private final int seconds;

    /**
     * Blank title or seconds <= 0 -> IllegalArgumentException.
     */
    public Song(String title, int seconds) {
        if (title == null || title.isBlank() || title.isEmpty()) {
            throw new IllegalArgumentException("title is not valid");
        }
        if (seconds <= 0) {
            throw new IllegalArgumentException("seconds cant be negative");
        }

        this.title = title;
        this.seconds = seconds;
    }

    public String title() {
        return this.title;
    }

    public int seconds() {
        return this.seconds;
    }

    /**
     * Blank title -> IllegalArgumentException.
     */
    public void rename(String title) {
        if (title == null || title.isBlank() || title.isEmpty()) {
            throw new IllegalArgumentException("title is not valid");
        }
        this.title = title;
    }

//    @Override
//    public boolean equals(Object obj) {
//        if (this == obj) return true;
//        if (obj == null || getClass() != obj.getClass()) return false;
//
//        Song other = (Song) obj;
//
//        return Objects.equals(title, other.title()) && Objects.equals(seconds, other.seconds());
//    }
//
//    @Override
//    public int hashCode() {
//        return Objects.hash(title, seconds);
//    }
}

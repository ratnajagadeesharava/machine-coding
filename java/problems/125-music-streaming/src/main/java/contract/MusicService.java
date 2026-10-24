package contract;

import java.util.List;
import java.util.Optional;

/**
 * Thin test entry point. Your design lives behind it. Must be thread-safe.
 *
 * <p>Every method throws IllegalArgumentException for an unknown user, song or playlist, for a null/blank string
 * argument, and for the other cases listed on the method. A failed call leaves the state unchanged.
 */
public interface MusicService {

    /** Adds a song to the catalogue. Error if the id already exists, any text is blank or durationSec is not positive. */
    void addSong(String songId, String title, String artist, int durationSec);

    /** Registers a user. Error if the id is already registered. */
    void registerUser(String userId);

    /**
     * Song ids matching the query, best first. Case-insensitive. A song matches when EVERY whitespace-separated query
     * token is a prefix of some whitespace-separated word of the title or of the artist name. Ranking: global play
     * count descending, then title ascending (case-insensitive), then song id ascending. At most k ids.
     * Error if the query is blank or k is not positive.
     */
    List<String> search(String query, int k);

    /** Counts one play of the song for the user: global count, the user's own count and the user's recent history. */
    void play(String userId, String songId);

    /** Total number of plays of the song by everybody. */
    long playCount(String songId);

    /** Number of times this user played this song. */
    long userPlayCount(String userId, String songId);

    /** The user's last distinct songs, most recent first, at most 10; replaying a song moves it to the front. */
    List<String> recentlyPlayed(String userId);

    /** The user likes the song. Idempotent. */
    void like(String userId, String songId);

    /** The user withdraws the like. Idempotent (also when there was no like). */
    void unlike(String userId, String songId);

    /** Number of distinct users currently liking the song. */
    int songLikes(String songId);

    /** Creates an empty playlist owned by the user and returns its id. Error if the user already has one with that name. */
    String createPlaylist(String userId, String name);

    /** Appends the song to the end of the playlist. Error if the song is already in it. */
    void addToPlaylist(String userId, String playlistId, String songId);

    /** Removes the song. Error if the song is not in the playlist. The other songs keep their order. */
    void removeFromPlaylist(String userId, String playlistId, String songId);

    /** Song ids in playlist order. A playlist is only visible to its owner: for anybody else it is unknown. */
    List<String> playlist(String userId, String playlistId);

    /** Deletes the playlist; its name becomes free again for that user. */
    void deletePlaylist(String userId, String playlistId);

    /** Appends the song to the end of the user's play queue (duplicates allowed). */
    void enqueue(String userId, String songId);

    /** Removes the oldest queued song and plays it (exactly like play). Empty queue: empty result, nothing changes. */
    Optional<String> playNext(String userId);

    /** Songs with at least one play, most played first (ties: title ascending case-insensitive, then id). At most k. Error if k is not positive. */
    List<String> topSongs(int k);

    /**
     * Songs by artists the user has played, that the user has never played. Artists are ranked by how many plays the
     * user has made of their songs (artist names compare case-insensitively, ignoring surrounding spaces). Songs rank
     * by that artist play count descending, then global play count descending, then song id ascending. At most k.
     * Error if k is not positive.
     */
    List<String> recommendations(String userId, int k);
}
